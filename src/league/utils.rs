use bevy::math::Mat4;

pub fn neg_mat_z(mat: &mut Mat4) {
    mat.w_axis.z = -mat.w_axis.z;
}
