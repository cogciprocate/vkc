use std::sync::Arc;
use std::ffi::CStr;
use std::ptr;
use std::path::Path;
use std::fs::File;
use std::io::{Read, BufReader};
use vk;
use ::{VkcResult, Device};


pub fn read_file<P: AsRef<Path>>(file: P) -> VkcResult<Vec<u8>> {
    let file_name = file.as_ref().display().to_string();
    let f = File::open(file).expect("shader file not found");
    let file_bytes = f.metadata().unwrap().len() as usize;
    let mut contents = Vec::<u8>::with_capacity(file_bytes);
    let mut reader = BufReader::new(f);
    match reader.read_to_end(&mut contents) {
        Ok(bytes) => {
            assert_eq!(bytes, file_bytes);
            println!("Read {} bytes from {}", bytes, &file_name);
        },
        Err(e) => panic!("{}", e),
    }
    Ok(contents)
}


/// Returns a column-major perspective matrix.
pub fn persp_matrix(width: u32, height: u32, fov_zoom: f32) -> [[f32; 4]; 4] {
    let zfar = 1024.0;
    let znear = 0.1;

    // let (width, height) = target.get_dimensions();
    let aspect_ratio = height as f32 / width as f32;
    let fov: f32 = 3.141592 / fov_zoom;
    let f = 1.0 / (fov / 2.0).tan();

    [
        [f *   aspect_ratio   ,    0.0,              0.0              ,   0.0],
        [         0.0         ,     f ,              0.0              ,   0.0],
        [         0.0         ,    0.0,  (zfar+znear)/(zfar-znear)    ,   1.0],
        [         0.0         ,    0.0, -(2.0*zfar*znear)/(zfar-znear),   0.0],
    ]
}

/// Returns a column-major view matrix.
pub fn view_matrix(position: &[f32; 3], direction: &[f32; 3], up: &[f32; 3]) -> [[f32; 4]; 4] {
    let f = {
        let f = direction;
        let len = f[0] * f[0] + f[1] * f[1] + f[2] * f[2];
        let len = len.sqrt();
        [f[0] / len, f[1] / len, f[2] / len]
    };

    let s = [up[1] * f[2] - up[2] * f[1],
             up[2] * f[0] - up[0] * f[2],
             up[0] * f[1] - up[1] * f[0]];

    let s_norm = {
        let len = s[0] * s[0] + s[1] * s[1] + s[2] * s[2];
        let len = len.sqrt();
        [s[0] / len, s[1] / len, s[2] / len]
    };

    let u = [f[1] * s_norm[2] - f[2] * s_norm[1],
             f[2] * s_norm[0] - f[0] * s_norm[2],
             f[0] * s_norm[1] - f[1] * s_norm[0]];

    let p = [-position[0] * s_norm[0] - position[1] * s_norm[1] - position[2] * s_norm[2],
             -position[0] * u[0] - position[1] * u[1] - position[2] * u[2],
             -position[0] * f[0] - position[1] * f[1] - position[2] * f[2]];

    [
        [s[0], u[0], f[0], 0.0],
        [s[1], u[1], f[1], 0.0],
        [s[2], u[2], f[2], 0.0],
        [p[0], p[1], p[2], 1.0],
    ]
}





        // ubo.model = glm::rotate(glm::mat4(1.0f), time * glm::radians(90.0f),
        //     glm::vec3(0.0f, 0.0f, 1.0f));

        // let axis = Unit::new_normalize(Vector3::new(0.0, 0.0, 1.0f32));
        // debug_assert_eq!(Unit::new_normalize(Vector3::z()), axis);
        // let angle = (time * (90.0f32)).to_radians();
        // let rotation = Rotation3::from_axis_angle(&axis, angle);
        // let rotation_matrix = Matrix4::<f32>::from_scaled_axis(rotation.scaled_axis());
        // let rotation_matrix_2 = Matrix4::<f32>::new_rotation(rotation.scaled_axis());
        // let rotation_matrix_3 = rotation.to_homogeneous();
        // let identity_matrix = Matrix4::<f32>::identity();
        // debug_assert_eq!(rotation_matrix, rotation_matrix_2);
        // debug_assert_eq!(rotation_matrix, rotation_matrix_3);
        // debug_assert_eq!(rotation_matrix * identity_matrix, rotation_matrix);


        // let model_rotation_angle = (time * (90.0f32)).to_radians();
        // // let model_rotation_angle = ((90.0f32)).to_radians();
        // let model_rotation_axis = Vector3::z();
        // let model_rotation_vector = model_rotation_angle * model_rotation_axis;
        // let model_translation_vector = nalgebra::zero::<Vector3<f32>>();
        // let model_isometry = Isometry3::new(model_translation_vector,
        //     model_rotation_vector);
        // let model_transformation_matrix: Matrix4<_> = model_isometry.to_homogeneous();

        // let eye = Point3::new(2.0, 2.0, 2.0f32);
        // let target = Point3::new(0.0, 0.0, 0.0);
        // let up = Vector3::y();
        // let view_isometry = Isometry3::look_at_rh(&eye, &target, &up);
        // let view_transformation_matrix = view_isometry.to_homogeneous();

        // let extent = self.swapchain.as_ref().unwrap().extent().clone();
        // debug_assert_eq!(PI / 4.0, 45.0f32.to_radians());
        // let projection_perspective = Perspective3::new(PI / 4.,
        //     extent.width as f32 / extent.height as f32, 0.1, 10.0f32);
        // let mut projection_matrix = projection_perspective.to_homogeneous();
        // // Flip sign because y is inverted in Vulkan.
        // projection_matrix[(1, 1)] *= -1.0;

        // let ubo = vkc::UniformBufferObject {
        //     model: model_transformation_matrix.into(),
        //     view: view_transformation_matrix.into(),
        //     proj: projection_matrix.into(),
        // };
