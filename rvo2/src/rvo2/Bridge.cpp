#include "Bridge.h"
#include "RVO.h"
#include "Vector2.h"

using namespace RVO;

RVOSimulatorHandle RVO_Create() {
    return static_cast<RVOSimulatorHandle>(new RVOSimulator());
}

size_t RVO_GetNumAgents(RVOSimulatorHandle simulator) {
    return static_cast<RVOSimulator*>(simulator)->getNumAgents();
}

void RVO_Delete(RVOSimulatorHandle simulator) {
    delete static_cast<RVOSimulator*>(simulator);
}

void RVO_SetTimeStep(RVOSimulatorHandle simulator, float timeStep) {
    static_cast<RVOSimulator*>(simulator)->setTimeStep(timeStep);
}

float RVO_GetTimeStep(RVOSimulatorHandle simulator) {
    return static_cast<RVOSimulator*>(simulator)->getTimeStep();
}

size_t RVO_AddAgent(RVOSimulatorHandle simulator,
                    const float* position,
                    float neighborDist,
                    size_t maxNeighbors,
                    float timeHorizon,
                    float timeHorizonObst,
                    float radius,
                    float maxSpeed,
                    const float* velocity) {
    RVOSimulator* sim = static_cast<RVOSimulator*>(simulator);
    return sim->addAgent(
        Vector2(position[0], position[1]),
        neighborDist,
        maxNeighbors,
        timeHorizon,
        timeHorizonObst,
        radius,
        maxSpeed,
        Vector2(velocity[0], velocity[1])
    );
}

void RVO_SetAgentPrefVelocity(RVOSimulatorHandle simulator, size_t agentNo, const float* velocity) {
    RVOSimulator* sim = static_cast<RVOSimulator*>(simulator);
    sim->setAgentPrefVelocity(agentNo, Vector2(velocity[0], velocity[1]));
}

void RVO_GetAgentPosition(RVOSimulatorHandle simulator, size_t agentNo, float* position) {
    RVOSimulator* sim = static_cast<RVOSimulator*>(simulator);
    Vector2 pos = sim->getAgentPosition(agentNo);
    position[0] = pos.x();
    position[1] = pos.y();
}

void RVO_GetAgentVelocity(RVOSimulatorHandle simulator, size_t agentNo, float* velocity) {
    RVOSimulator* sim = static_cast<RVOSimulator*>(simulator);
    Vector2 vel = sim->getAgentVelocity(agentNo);
    velocity[0] = vel.x();
    velocity[1] = vel.y();
}

void RVO_DoStep(RVOSimulatorHandle simulator) {
    static_cast<RVOSimulator*>(simulator)->doStep();
}

void RVO_AddObstacle(RVOSimulatorHandle simulator, const float* vertices, size_t numVertices) {
    RVOSimulator* sim = static_cast<RVOSimulator*>(simulator);
    std::vector<Vector2> obstacleVertices;
    for (size_t i = 0; i < numVertices; ++i) {
        obstacleVertices.push_back(Vector2(vertices[2*i], vertices[2*i + 1]));
    }
    sim->addObstacle(obstacleVertices);
}

void RVO_ProcessObstacles(RVOSimulatorHandle simulator) {
    static_cast<RVOSimulator*>(simulator)->processObstacles();
} 