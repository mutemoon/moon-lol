AnimationState - Component
LOLAnimationGraph - Asset
LOLAnimationGraphHandler - Component

AnimationPlayer - Component
AnimationGraph - Asset
AnimationGraphHandler - Component

AnimationState

lol 中记录当前播放的动画名

LOLAnimationGraph

lol 动画图

LOLAnimationGraphHandler

lol 动画图句柄

AnimationPlayer

bevy 动画播放器，记录了当前动画的状态

AnimationGraph

bevy 动画图，存放了很多剪辑节点 Handle<AnimationClip> 及其他节点

AnimationGraphHandler

bevy 动画图句柄

- (Character, WorldAssetRoot, WorldInstance) Character Entity
  - (Name) Gltf World Root Entity
    - (AnimationPlayer, AnimatedBy, AnimationTargetId) Root Bone Node Entity 1
      - (AnimatedBy, AnimationTargetId) Bone Node Entity 1
        - ...
      - (AnimatedBy, AnimationTargetId) Bone Node Entity 2
        - ...
    - (AnimationPlayer, AnimatedBy, AnimationTargetId) Gltf Root Bone Node Entity 2
      - (AnimatedBy, AnimationTargetId) Bone Node Entity 1
        - ...
      - (AnimatedBy, AnimationTargetId) Bone Node Entity 2
        - ...

skin.ron

"WorldAssetRoot": Path("characters/{character_name}/skins/skin0.glb"),
"LOLAnimationGraphHandler": Path("characters/{character_name}/animations/skin0.ron"),

LOLAnimationGraphSerialized (gltf_path, clip_name) -> AnimationGraph (Handle<AnimationClip>) + LOLAnimationGraph (AnimationNodeIndex)

AnimationGraph -> Root Bone Node
