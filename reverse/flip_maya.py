import maya.cmds as cmds
import maya.api.OpenMaya as om

# 获取选中物体
sel = cmds.ls(selection=True)
if not sel:
    raise RuntimeError("请先选择一个物体")

# 创建交换Y和Z轴的矩阵
swap_matrix = [
    -1, 0, 0, 0,  # X轴保持不变
    0, 0, 1, 0,  # Y轴变为原来的Z轴
    0, 1, 0, 0,  # Z轴变为原来的Y轴
    0, 0, 0, 1
]

# 将列表转换为MMatrix
matrix = om.MMatrix(swap_matrix)

# 应用矩阵到选中物体
for obj in sel:
    # 获取物体的世界矩阵
    world_matrix = cmds.xform(obj, query=True, matrix=True, worldSpace=True)
    
    # 转换为MMatrix并相乘
    current_matrix = om.MMatrix(world_matrix)
    result_matrix = current_matrix * matrix
    
    # 应用新矩阵
    cmds.xform(obj, matrix=list(result_matrix), worldSpace=True)