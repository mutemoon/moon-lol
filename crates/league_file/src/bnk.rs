//! Wwise SoundBank（.bnk）解析。
//!
//! 移植自 Morilli/bnk-extract（LoL 专用、经实战验证）的分段与字段布局。
//! 仅解析音效映射所需的部分：
//! - `DIDX`/`DATA`：媒体索引与原始 wem 字节（音频 bank）
//! - `HIRC`：对象层级——Event(4)/EventAction(3)/Sound(2)/RandomContainer(5)/SwitchContainer(6)
//!
//! 事件名经 32bit FNV-1（[`league_utils::hash_wwise`]）得到 Event 的 self_id；
//! Event → EventAction(play) → 目标对象 → 递归容器 → Sound.file_id（即 wem id）。
//!
//! 音乐相关对象（10~13）与 SFX 无关，直接按对象长度跳过。

use std::collections::HashMap;

use league_utils::hash_wwise;

/// 只读游标，越界读取返回 0 并停在末尾，避免 panic；单对象解析出错不影响其它对象
/// （外层循环按对象长度重新对齐）。
struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Reader { buf, pos: 0 }
    }

    fn tell(&self) -> usize {
        self.pos
    }

    fn seek(&mut self, pos: usize) {
        self.pos = pos.min(self.buf.len());
    }

    fn skip(&mut self, n: usize) {
        self.pos = self.pos.saturating_add(n).min(self.buf.len());
    }

    fn u8(&mut self) -> u8 {
        let v = self.buf.get(self.pos).copied().unwrap_or(0);
        self.pos = (self.pos + 1).min(self.buf.len());
        v
    }

    fn u16(&mut self) -> u16 {
        let mut b = [0u8; 2];
        for k in 0..2 {
            b[k] = self.buf.get(self.pos + k).copied().unwrap_or(0);
        }
        self.skip(2);
        u16::from_le_bytes(b)
    }

    fn u32(&mut self) -> u32 {
        let mut b = [0u8; 4];
        for k in 0..4 {
            b[k] = self.buf.get(self.pos + k).copied().unwrap_or(0);
        }
        self.skip(4);
        u32::from_le_bytes(b)
    }

    fn u32s(&mut self, count: usize) -> Vec<u32> {
        // 限制单次分配，避免损坏数据导致的巨额分配
        let count = count.min(4096);
        (0..count).map(|_| self.u32()).collect()
    }
}

#[derive(Debug, Clone)]
struct Sound {
    file_id: u32,
}

#[derive(Debug, Clone)]
struct EventAction {
    action_type: u8,
    target_id: u32,
}

#[derive(Debug, Clone)]
struct Event {
    action_ids: Vec<u32>,
}

#[derive(Debug, Clone)]
struct Container {
    /// RandomContainer 的 sound_ids 或 SwitchContainer 的 children。
    children: Vec<u32>,
}

/// 单个 bnk 文件解析结果，可能同时含媒体与 HIRC。
#[derive(Debug, Default)]
pub struct Bnk {
    pub version: u32,
    /// wem id -> 原始 wem 字节（来自 DIDX/DATA）。
    pub media: HashMap<u32, Vec<u8>>,
    events: HashMap<u32, Event>,
    event_actions: HashMap<u32, EventAction>,
    sounds: HashMap<u32, Sound>,
    containers: HashMap<u32, Container>,
}

impl Bnk {
    pub fn parse(input: &[u8]) -> Option<Self> {
        if input.len() < 8 || &input[0..4] != b"BKHD" {
            return None;
        }
        let mut bnk = Bnk::default();

        // BKHD: tag(4) size(4) version(4) bank_id(4) ...
        let bkhd_size = u32::from_le_bytes([input[4], input[5], input[6], input[7]]) as usize;
        if bkhd_size >= 4 {
            bnk.version = u32::from_le_bytes([input[8], input[9], input[10], input[11]]);
        }

        bnk.parse_media(input);
        bnk.parse_hirc(input);
        Some(bnk)
    }

    /// 定位一个顶层 chunk，返回 (数据起始偏移, 数据长度)。
    fn find_chunk(input: &[u8], tag: &[u8; 4]) -> Option<(usize, usize)> {
        let mut pos = 0usize;
        while pos + 8 <= input.len() {
            let this_tag = &input[pos..pos + 4];
            let size = u32::from_le_bytes([
                input[pos + 4],
                input[pos + 5],
                input[pos + 6],
                input[pos + 7],
            ]) as usize;
            let data_start = pos + 8;
            if this_tag == tag {
                return Some((data_start, size));
            }
            pos = data_start.saturating_add(size);
        }
        None
    }

    fn parse_media(&mut self, input: &[u8]) {
        let Some((didx_start, didx_size)) = Self::find_chunk(input, b"DIDX") else {
            return;
        };
        let Some((data_start, data_size)) = Self::find_chunk(input, b"DATA") else {
            return;
        };
        let entry_count = didx_size / 12;
        for k in 0..entry_count {
            let base = didx_start + k * 12;
            if base + 12 > input.len() {
                break;
            }
            let id = u32::from_le_bytes([input[base], input[base + 1], input[base + 2], input[base + 3]]);
            let off = u32::from_le_bytes([input[base + 4], input[base + 5], input[base + 6], input[base + 7]]) as usize;
            let len = u32::from_le_bytes([input[base + 8], input[base + 9], input[base + 10], input[base + 11]]) as usize;
            let start = data_start + off;
            let end = start + len;
            if end <= input.len() && end <= data_start + data_size {
                self.media.insert(id, input[start..end].to_vec());
            }
        }
    }

    fn parse_hirc(&mut self, input: &[u8]) {
        let Some((hirc_start, hirc_size)) = Self::find_chunk(input, b"HIRC") else {
            return;
        };
        let version = self.version;
        let mut r = Reader::new(input);
        r.seek(hirc_start);
        let _num_objects = r.u32();
        let end = hirc_start + hirc_size;
        while r.tell() < end {
            let obj_type = r.u8();
            let obj_len = r.u32() as usize;
            let obj_start = r.tell();
            match obj_type {
                2 => self.read_sound(&mut r, version),
                3 => self.read_event_action(&mut r),
                4 => self.read_event(&mut r, version),
                5 => self.read_random_container(&mut r, version),
                6 => self.read_switch_container(&mut r, version),
                _ => {}
            }
            r.seek(obj_start + obj_len);
        }
    }

    fn read_sound(&mut self, r: &mut Reader, version: u32) {
        let self_id = r.u32();
        r.skip(4); // plugin id
        let _is_streamed = r.u8();
        if version <= 0x59 {
            r.skip(3);
        }
        if version <= 0x70 {
            r.skip(4);
        }
        let file_id = r.u32();
        self.sounds.insert(self_id, Sound { file_id });
    }

    fn read_event_action(&mut self, r: &mut Reader) {
        let self_id = r.u32();
        let _scope = r.u8();
        let action_type = r.u8();
        // 仅关注 play(4)；set switch(25)/set state(18) 属音乐逻辑，此处不取目标。
        let target_id = if action_type == 25 || action_type == 18 {
            0
        } else {
            r.u32()
        };
        self.event_actions.insert(
            self_id,
            EventAction {
                action_type,
                target_id,
            },
        );
    }

    fn read_event(&mut self, r: &mut Reader, version: u32) {
        let self_id = r.u32();
        let amount = r.u8() as usize;
        if version == 0x58 {
            r.skip(3);
        }
        let action_ids = r.u32s(amount);
        self.events.insert(self_id, Event { action_ids });
    }

    fn read_random_container(&mut self, r: &mut Reader, version: u32) {
        let self_id = r.u32();
        skip_base_params(r, version);
        r.skip(24);
        let amount = r.u32() as usize;
        let children = r.u32s(amount);
        self.containers.insert(self_id, Container { children });
    }

    fn read_switch_container(&mut self, r: &mut Reader, version: u32) {
        let self_id = r.u32();
        skip_base_params(r, version);
        let _group_type = r.u8();
        if version <= 0x59 {
            r.skip(3);
        }
        let _group_id = r.u32();
        r.skip(5);
        let amount = r.u32() as usize;
        let children = r.u32s(amount);
        self.containers.insert(self_id, Container { children });
    }

    /// 递归把一个对象 id 解析成一批 wem id（穿过随机/切换容器直到 Sound）。
    fn resolve(&self, id: u32, out: &mut Vec<u32>, depth: u32) {
        if depth > 32 {
            return;
        }
        if let Some(c) = self.containers.get(&id) {
            for &child in &c.children {
                self.resolve(child, out, depth + 1);
            }
            return;
        }
        if let Some(s) = self.sounds.get(&id) {
            out.push(s.file_id);
        }
    }

    /// 把一个事件名解析成关联的 wem id 列表（去重、保序）。
    pub fn resolve_event(&self, event_name: &str) -> Vec<u32> {
        let hash = hash_wwise(event_name);
        let Some(event) = self.events.get(&hash) else {
            return Vec::new();
        };
        let mut wems = Vec::new();
        for action_id in &event.action_ids {
            if let Some(action) = self.event_actions.get(action_id) {
                if action.action_type == 4 {
                    self.resolve(action.target_id, &mut wems, 0);
                }
            }
        }
        wems.dedup();
        wems
    }

    pub fn has_events(&self) -> bool {
        !self.events.is_empty()
    }

    /// 鍒涘缓鐨勪簨浠舵暟銆?
    pub fn event_count(&self) -> usize {
        self.events.len()
    }
}

// ---- 以下为 skip_base_params 及其子过程，忠实移植自 bnk-extract ----

fn skip_initial_fx_params(r: &mut Reader, version: u32) {
    r.skip(1);
    let num_fx = r.u8();
    if num_fx != 0 {
        r.skip(1);
    }
    r.skip(num_fx as usize * if version <= 0x91 { 7 } else { 6 });
}

fn skip_initial_params(r: &mut Reader) {
    let prop_count = r.u8();
    r.skip(5 * prop_count as usize);
    let prop_count = r.u8();
    r.skip(9 * prop_count as usize);
}

fn skip_positioning_params(r: &mut Reader, version: u32) {
    let positioning_bits = r.u8();
    let has_positioning = positioning_bits & 1 != 0;
    let mut has_3d = false;
    let mut has_automation = false;
    if has_positioning {
        if version <= 0x59 {
            let has_2d = r.u8() != 0;
            has_3d = r.u8() != 0;
            if has_2d {
                r.u8();
            }
        } else {
            has_3d = positioning_bits & 0x2 != 0;
        }
    }
    if has_positioning && has_3d {
        if version <= 0x59 {
            has_automation = (r.u8() & 3) != 1;
            r.skip(8);
        } else {
            has_automation = ((positioning_bits >> 5) & 3) != 0;
            r.u8();
        }
    }
    if has_automation {
        r.skip(if version <= 0x59 { 9 } else { 5 });
        let num_vertices = r.u32() as usize;
        r.skip(16 * num_vertices);
        let num_playlist_items = r.u32() as usize;
        r.skip(if version <= 0x59 { 16 } else { 20 } * num_playlist_items);
    } else if version <= 0x59 {
        r.u8();
    }
}

fn skip_aux_params(r: &mut Reader, version: u32) {
    let has_aux = (r.u8() >> 3) & 1 != 0;
    if has_aux {
        r.skip(4 * 4);
    }
    if version > 0x87 {
        r.skip(4);
    }
}

fn skip_rtpc(r: &mut Reader, version: u32) {
    let num_rtpc = r.u16();
    for _ in 0..num_rtpc {
        r.skip(if version <= 0x59 { 13 } else { 12 });
        let point_count = r.u16();
        r.skip(12 * point_count as usize);
    }
}

/// 跳过 CAkBankSourceData 基础参数段，返回后游标停在派生对象的自有字段处。
fn skip_base_params(r: &mut Reader, version: u32) {
    skip_initial_fx_params(r, version);
    if version > 0x88 {
        r.skip(1);
        let num_fx = r.u8();
        r.skip(6 * num_fx as usize);
    }
    if version > 0x59 && version <= 0x91 {
        r.skip(1);
    }
    let _bus_id = r.u32();
    let _parent_id = r.u32();
    r.skip(if version <= 0x59 { 2 } else { 1 });

    skip_initial_params(r);
    skip_positioning_params(r, version);
    skip_aux_params(r, version);

    r.skip(6);

    let state_props = r.u8();
    r.skip(3 * state_props as usize);
    let state_groups = r.u8();
    for _ in 0..state_groups {
        r.skip(5);
        let states = r.u8();
        r.skip(8 * states as usize);
    }

    skip_rtpc(r, version);
}
