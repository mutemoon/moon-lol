#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::os::raw::{c_float, c_void};

#[repr(C)]
pub struct RVOSimulator {
    _private: [u8; 0],
}

extern "C" {
    pub fn RVO_Create() -> *mut c_void;
    pub fn RVO_GetNumAgents(simulator: *mut c_void) -> usize;
    pub fn RVO_Delete(simulator: *mut c_void);
    pub fn RVO_SetTimeStep(simulator: *mut c_void, timeStep: c_float);
    pub fn RVO_GetTimeStep(simulator: *mut c_void) -> c_float;
    pub fn RVO_AddAgent(
        simulator: *mut c_void,
        position: *const c_float,
        neighborDist: c_float,
        maxNeighbors: usize,
        timeHorizon: c_float,
        timeHorizonObst: c_float,
        radius: c_float,
        maxSpeed: c_float,
        velocity: *const c_float,
    ) -> usize;
    pub fn RVO_SetAgentPrefVelocity(
        simulator: *mut c_void,
        agentNo: usize,
        velocity: *const c_float,
    );
    pub fn RVO_GetAgentPosition(simulator: *mut c_void, agentNo: usize, position: *mut c_float);
    pub fn RVO_GetAgentVelocity(simulator: *mut c_void, agentNo: usize, velocity: *mut c_float);
    pub fn RVO_DoStep(simulator: *mut c_void);
    pub fn RVO_AddObstacle(simulator: *mut c_void, vertices: *const c_float, numVertices: usize);
    pub fn RVO_ProcessObstacles(simulator: *mut c_void);
}

// 安全的Rust包装器
pub struct RVOSimulatorWrapper {
    simulator: *mut c_void,
}

impl RVOSimulatorWrapper {
    pub fn new() -> Self {
        let simulator = unsafe { RVO_Create() };
        RVOSimulatorWrapper { simulator }
    }

    pub fn get_agents_num(&self) -> usize {
        unsafe { RVO_GetNumAgents(self.simulator) }
    }

    pub fn set_time_step(&mut self, time_step: f32) {
        unsafe { RVO_SetTimeStep(self.simulator, time_step) }
    }

    pub fn get_time_step(&self) -> f32 {
        unsafe { RVO_GetTimeStep(self.simulator) }
    }

    pub fn add_agent(
        &mut self,
        position: &[f32; 2],
        neighbor_dist: f32,
        max_neighbors: usize,
        time_horizon: f32,
        time_horizon_obst: f32,
        radius: f32,
        max_speed: f32,
        velocity: &[f32; 2],
    ) -> usize {
        unsafe {
            RVO_AddAgent(
                self.simulator,
                position.as_ptr(),
                neighbor_dist,
                max_neighbors,
                time_horizon,
                time_horizon_obst,
                radius,
                max_speed,
                velocity.as_ptr(),
            )
        }
    }

    pub fn set_agent_pref_velocity(&mut self, agent_no: usize, velocity: &[f32; 2]) {
        unsafe {
            RVO_SetAgentPrefVelocity(self.simulator, agent_no, velocity.as_ptr());
        }
    }

    pub fn get_agent_position(&self, agent_no: usize) -> [f32; 2] {
        let mut position = [0.0; 2];
        unsafe {
            RVO_GetAgentPosition(self.simulator, agent_no, position.as_mut_ptr());
        }
        position
    }

    pub fn get_agent_velocity(&self, agent_no: usize) -> [f32; 2] {
        let mut velocity = [0.0; 2];
        unsafe {
            RVO_GetAgentVelocity(self.simulator, agent_no, velocity.as_mut_ptr());
        }
        velocity
    }

    pub fn do_step(&mut self) {
        unsafe { RVO_DoStep(self.simulator) }
    }

    pub fn add_obstacle(&mut self, vertices: &[[f32; 2]]) {
        let flat_vertices: Vec<f32> = vertices.iter().flat_map(|v| v.iter().cloned()).collect();
        unsafe {
            RVO_AddObstacle(self.simulator, flat_vertices.as_ptr(), vertices.len());
        }
    }

    pub fn process_obstacles(&mut self) {
        unsafe { RVO_ProcessObstacles(self.simulator) }
    }
}

impl Drop for RVOSimulatorWrapper {
    fn drop(&mut self) {
        unsafe { RVO_Delete(self.simulator) }
    }
}

// 为了安全起见，实现Send和Sync
unsafe impl Send for RVOSimulatorWrapper {}
unsafe impl Sync for RVOSimulatorWrapper {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_create_simulator() {
        let sim = RVOSimulatorWrapper::new();
        assert!(!sim.simulator.is_null());
    }

    #[test]
    fn test_time_step() {
        let mut sim = RVOSimulatorWrapper::new();
        let time_step = 0.25;
        sim.set_time_step(time_step);
        assert_eq!(sim.get_time_step(), time_step);
    }

    #[test]
    fn test_add_agent() {
        let mut sim = RVOSimulatorWrapper::new();
        let position = [0.0, 0.0];
        let velocity = [1.0, 0.0];

        let agent_id = sim.add_agent(
            &position, 15.0, // neighbor_dist
            10,   // max_neighbors
            5.0,  // time_horizon
            5.0,  // time_horizon_obst
            2.0,  // radius
            2.0,  // max_speed
            &velocity,
        );

        assert_eq!(agent_id, 0); // 第一个代理的ID应该是0

        // 验证代理位置
        let pos = sim.get_agent_position(agent_id);
        assert_eq!(pos, position);

        // 验证代理速度
        let vel = sim.get_agent_velocity(agent_id);
        assert_eq!(vel, velocity);
    }

    #[test]
    fn test_multiple_agents() {
        let mut sim = RVOSimulatorWrapper::new();

        // 添加四个代理，形成一个正方形
        let positions = [[-5.0, -5.0], [-5.0, 5.0], [5.0, -5.0], [5.0, 5.0]];

        let mut agent_ids = Vec::new();
        for pos in positions.iter() {
            let agent_id = sim.add_agent(
                pos,
                15.0,        // neighbor_dist
                10,          // max_neighbors
                5.0,         // time_horizon
                5.0,         // time_horizon_obst
                1.0,         // radius
                2.0,         // max_speed
                &[0.0, 0.0], // 初始速度为0
            );
            agent_ids.push(agent_id);
        }

        // 验证所有代理的位置
        for (i, expected_pos) in positions.iter().enumerate() {
            let pos = sim.get_agent_position(agent_ids[i]);
            assert_eq!(pos, *expected_pos);
        }
    }

    #[test]
    fn test_add_obstacle() {
        let mut sim = RVOSimulatorWrapper::new();

        // 创建一个三角形障碍物
        let obstacle = vec![[0.0, 0.0], [2.0, 0.0], [1.0, 2.0]];

        sim.add_obstacle(&obstacle);
        sim.process_obstacles();

        // 添加一个代理在障碍物附近
        let agent_id = sim.add_agent(
            &[-1.0, -1.0], // 位置
            15.0,          // neighbor_dist
            10,            // max_neighbors
            5.0,           // time_horizon
            5.0,           // time_horizon_obst
            0.5,           // radius
            2.0,           // max_speed
            &[1.0, 1.0],   // 朝向障碍物的速度
        );

        // 运行几步模拟
        for _ in 0..10 {
            sim.do_step();
        }

        // 获取代理的最终位置，确保它没有穿过障碍物
        let final_pos = sim.get_agent_position(agent_id);
        // 简单检查代理是否还在障碍物外部
        assert!(final_pos[0] < 0.0 || final_pos[1] < 0.0);
    }

    #[test]
    fn test_agent_preferred_velocity() {
        let mut sim = RVOSimulatorWrapper::new();

        // 添加一个代理
        let agent_id = sim.add_agent(
            &[0.0, 0.0], // 位置
            15.0,        // neighbor_dist
            10,          // max_neighbors
            5.0,         // time_horizon
            5.0,         // time_horizon_obst
            1.0,         // radius
            2.0,         // max_speed
            &[0.0, 0.0], // 初始速度
        );

        // 设置首选速度
        let pref_velocity = [1.0, 1.0];
        sim.set_agent_pref_velocity(agent_id, &pref_velocity);

        // 运行几步模拟
        sim.do_step();

        // 获取代理的实际速度
        let actual_velocity = sim.get_agent_velocity(agent_id);

        // 由于没有障碍物和其他代理，实际速度应该接近首选速度
        // 考虑到浮点数精度，使用近似比较
        assert!((actual_velocity[0] - pref_velocity[0]).abs() < 0.1);
        assert!((actual_velocity[1] - pref_velocity[1]).abs() < 0.1);
    }
}
