export interface WebSystemSummaryItem {
  name: string
  hash: number
  hash_hex: string
  emitter_count: usize
}

export interface WebHeroSummary {
  id: string
  name: string
  category: 'five_tigers' | 'four_sisters' | 'other' | string
  count: number
  systems: WebSystemSummaryItem[]
}

export interface Vec2 {
  x: number
  y: number
}

export interface Vec3 {
  x: number
  y: number
  z: number
}

export interface Vec4 {
  x: number
  y: number
  z: number
  w: number
}

export interface SamplerValue<T> {
  base_sampler: T | { Constant: T } | { Curve: { samples: [number, T][] } }
  prob_curves?: any[]
}

export interface VfxTexture {
  path: string
}

export interface ConfigVfxAlphaErosionDefinition {
  erosion_drive_curve?: any
  erosion_map?: VfxTexture
  erosion_map_channel_mixer?: any
  erosion_slice_width?: number
}

export interface ConfigVfxEmitterDefinition {
  emitter_name?: string
  lifetime?: number
  rate?: any
  birth_acceleration?: any
  birth_color?: any
  birth_rotation0?: any
  birth_scale0?: any
  birth_uv_offset?: any
  birth_uv_scroll_rate?: any
  birth_velocity?: any
  color?: any
  scale0?: any
  particle_lifetime?: any
  emitter_position?: any
  blend_mode?: number
  alpha_ref?: number
  primitive?: string | any
  is_single_particle?: boolean
  is_direction_oriented?: boolean
  texture?: VfxTexture
  particle_color_texture?: VfxTexture
  alpha_erosion_definition?: ConfigVfxAlphaErosionDefinition
  color_look_up_type_y?: number
  [key: string]: any
}

export interface ConfigVfxSystemDefinition {
  particle_name: string
  particle_path: string
  complex_emitter_definition_data?: ConfigVfxEmitterDefinition[]
  simple_emitter_definition_data?: ConfigVfxEmitterDefinition[]
  [key: string]: any
}

export interface WebParticleSystem {
  name: string
  hash: number
  hash_hex: string
  emitter_count: number
  ron_str: string
  def: ConfigVfxSystemDefinition
}

export interface WebHeroDetail {
  hero_id: string
  systems: WebParticleSystem[]
}

type usize = number
