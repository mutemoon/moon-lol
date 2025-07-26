import bpy
import mathutils

# 获取选中物体
sel = bpy.context.selected_objects
if not sel:
    raise RuntimeError("请先选择一个物体")

# 创建交换Y和Z轴的矩阵
swap_matrix = mathutils.Matrix((
    (-1, 0, 0, 0),  # X轴保持不变
    (0, 0, 1, 0),   # Y轴变为原来的Z轴
    (0, 1, 0, 0),   # Z轴变为原来的Y轴
    (0, 0, 0, 1)
))

# 应用矩阵到选中物体
for obj in sel:
    # 获取物体的世界矩阵
    world_matrix = obj.matrix_world
    
    # 矩阵相乘
    result_matrix = world_matrix @ swap_matrix
    
    # 应用新矩阵
    obj.matrix_world = result_matrix 