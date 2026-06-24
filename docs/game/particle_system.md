
# 需求

- 粒子发射器和粒子的生命由生命周期组件统一管理，实体生命结束时会立即销毁实体

- 粒子发射器生命结束时，粒子可能仍需要存活

- 英雄生命结束时，所有粒子发射器和粒子都需要销毁

- 粒子的 transform 应该是在粒子发射器的 transform 下，而粒子发射器的 transform 应该在英雄下

# 方案

## 层级结构

entity
└── emitter ParticleId
    ├── particle1
    └── particle2 ParticleDecal(Hash<Entity, Entity>) MeshMaterial3d<ParticleMaterialUnlitDecal>

map
└── geometry MapGeometry Mesh3d

decal_geometry Mesh3d MeshMaterial3d<ParticleMaterialUnlitDecal>

- 生命周期支持生命结束时不销毁，当没有子实体时才销毁的模式，emitter 应用这种模式

- 通过 ParticleId 销毁指定的 emitter，子粒子自动销毁

- emitter 的 is_local_orientation 为 true 时，emitter 手动更新自己的 GlobalTransform，particle 使用父实体即 emitter 手动更新后的 GlobalTransform 计算 world matrix 传给 shader

- 遍历 (Entity, ParticleDecal) 实体，取 ParticleDecalGeometry
