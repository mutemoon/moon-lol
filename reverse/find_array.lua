-- 读取玩家index
local function read_player_index(player_addr)
    return readSmallInteger(player_addr + 0x10)
end

-- 验证数组地址是否有效
local function verify_array_addr(array_addr, index)
    local next_obj_addr = readQword(array_addr + 0x8 * (index + 1))
    if next_obj_addr == 0 then return false end

    local next_index = readSmallInteger(next_obj_addr + 0x10)
    return next_index == index + 1
end

-- 主搜索函数
function find_object_array(player_addr)
    local player_index = read_player_index(player_addr)

    if player_index == nil or player_index == 0 then
        return nil
    end

    -- 搜索包含player_addr的指针
    -- 将8字节地址转换为字节序列
    local addr_bytes = string.format("%02X %02X %02X %02X %02X %02X %02X %02X",
        player_addr & 0xFF,
        (player_addr >> 8) & 0xFF,
        (player_addr >> 16) & 0xFF,
        (player_addr >> 24) & 0xFF,
        (player_addr >> 32) & 0xFF,
        (player_addr >> 40) & 0xFF,
        (player_addr >> 48) & 0xFF,
        (player_addr >> 56) & 0xFF
    )

    local array_addr_array = AOBScan(addr_bytes)
    
    -- 检查并打印 array_addr_array 的基本属性
    if array_addr_array then
        print("Count:", array_addr_array.Count)  -- 结果数量
        
        -- 遍历并打印每个搜索结果的信息
        for i=1, array_addr_array.Count do
            local array_addr_str = array_addr_array[i-1]  -- 注意：索引从0开始
            local array_addr = tonumber(array_addr_str, 16)
            local potential_array_addr = array_addr - (player_index * 8)

            -- 验证这个地址是否为真正的数组地址
            if verify_array_addr(potential_array_addr, player_index) then
                array_addr_array.destroy()
                return potential_array_addr
            end
        end
        
        -- 使用完后记得销毁
        array_addr_array.destroy()
    else
        print("未找到结果")
    end

    return nil
end

local res = find_object_array(readQword("League of Legends.exe + 1AEEFA0"))

print(res)