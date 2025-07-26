#ifndef RVO_BRIDGE_H_
#define RVO_BRIDGE_H_

#include "RVO.h"
#include <memory>

#ifdef __cplusplus
extern "C" {
#endif

// 不透明指针类型，用于在Rust中表示RVOSimulator
typedef void* RVOSimulatorHandle;

// 创建新的RVO模拟器实例
RVOSimulatorHandle RVO_Create();

// 获取代理数量
size_t RVO_GetNumAgents(RVOSimulatorHandle simulator);

// 删除RVO模拟器实例
void RVO_Delete(RVOSimulatorHandle simulator);

// 设置时间步长
void RVO_SetTimeStep(RVOSimulatorHandle simulator, float timeStep);

// 获取时间步长
float RVO_GetTimeStep(RVOSimulatorHandle simulator);

// 添加代理
size_t RVO_AddAgent(RVOSimulatorHandle simulator, 
                    const float* position,
                    float neighborDist,
                    size_t maxNeighbors,
                    float timeHorizon,
                    float timeHorizonObst,
                    float radius,
                    float maxSpeed,
                    const float* velocity);

// 设置代理的首选速度
void RVO_SetAgentPrefVelocity(RVOSimulatorHandle simulator, size_t agentNo, const float* velocity);

// 获取代理的位置
void RVO_GetAgentPosition(RVOSimulatorHandle simulator, size_t agentNo, float* position);

// 获取代理的速度
void RVO_GetAgentVelocity(RVOSimulatorHandle simulator, size_t agentNo, float* velocity);

// 执行模拟步骤
void RVO_DoStep(RVOSimulatorHandle simulator);

// 添加障碍物
void RVO_AddObstacle(RVOSimulatorHandle simulator, const float* vertices, size_t numVertices);

// 处理障碍物
void RVO_ProcessObstacles(RVOSimulatorHandle simulator);

#ifdef __cplusplus
}
#endif

#endif // RVO_BRIDGE_H_ 