use std::{path::PathBuf, sync::Arc};

use crate::camera::Camera;

#[derive(Default, Debug, Copy, Clone)]
pub enum VertexType {
    #[default]
    Split,
    Combined,
}

#[derive(Debug, Clone, Default)]
pub struct SceneDescriptor {
    pub name: String,
    pub shader: PathBuf,
    pub vertex_type: VertexType,
    pub model: Option<PathBuf>,
    pub camera: Camera,
    pub res: (u32, u32),
}

pub fn get_scenes() -> Arc<[SceneDescriptor]> {
    let basic_scene_camera = Camera {
        eye: (2.0, 1.5, 2.0).into(),
        target: (0.0, 0.5, 0.0).into(),
        up: (0.0, 1.0, 0.0).into(), 
        constant: 1.0,
        ..Default::default()
    };

    let utah_teapot_camera = Camera {
        eye: (0.15, 1.5, 10.0).into(),
        target: (0.15, 1.5, 0.0).into(),
        up: (0.0, 1.0, 0.0).into(), 
        constant: 2.5,
        ..Default::default()
    };

    let cornell_box_camera = Camera {
        eye: (277.0, 275.0, -570.0).into(),
        target: (277.0, 275.0, 0.0).into(),
        up: (0.0, 1.0, 0.0).into(), 
        constant: 1.0,
        ..Default::default()
    };

    let bunny_camera = Camera {
        eye: (-0.02, 0.11, 0.6).into(),
        target: (-0.02, 0.11, 0.0).into(),
        up: (0.0, 1.0, 0.0).into(), 
        constant: 3.5,
        ..Default::default()
    };

    let cornell_box_path = PathBuf::from("res/models/CornellBox.obj");
    let cornell_box_with_blocks_path = PathBuf::from("res/models/CornellBoxWithBlocks.obj");
    let bunny_path = PathBuf::from("res/models/bunny.obj");
    let teapot_path = PathBuf::from("res/models/teapot.obj");

    Arc::new([
        SceneDescriptor {
            name: String::from("Default"),
            shader: PathBuf::from("res/shaders/shader.wgsl"),
            model: Some(teapot_path.clone()),
            camera: utah_teapot_camera.clone(),
            res: (512, 512),
            ..Default::default()
        },
        //SceneDescriptor {
        //    name: String::from("Worksheet 1"),
        //    shader: PathBuf::from("res/shaders/worksheet1.wgsl"),
        //    model: None,
        //    camera: basic_scene_camera.clone(),
        //    res: (512, 512),
        //},
        SceneDescriptor {
            name: String::from("Worksheet 2"),
            shader: PathBuf::from("res/shaders/worksheet2.wgsl"),
            model: None,
            camera: basic_scene_camera.clone(),
            res: (512, 512),
            ..Default::default()
        },
        SceneDescriptor {
            name: String::from("Worksheet 3"),
            shader: PathBuf::from("res/shaders/worksheet3.wgsl"),
            model: None,
            camera: basic_scene_camera.clone(),
            res: (512, 512),
            ..Default::default()
        },
        SceneDescriptor {
            name: String::from("W5 Teapot"),
            shader: PathBuf::from("res/shaders/w05_teapot.wgsl"),
            model: Some(teapot_path.clone()),
            camera: utah_teapot_camera.clone(),
            res: (800, 450),
            ..Default::default()
        },
        SceneDescriptor {
            name: String::from("W5 Cornell Box"),
            shader: PathBuf::from("res/shaders/w05_cornell_box.wgsl"),
            model: Some(cornell_box_path.clone()),
            camera: cornell_box_camera.clone(),
            res: (512, 512),
            ..Default::default()
        },
        //SceneDescriptor {
        //    name: String::from("W6 Bunny"),
        //    shader: PathBuf::from("res/shaders/w06_bunny.wgsl"),
        //    model: Some(bunny_path.clone()),
        //    camera: bunny_camera.clone(),
        //    res: (512, 512),
        //},
        //SceneDescriptor {
        //    name: String::from("W6 Cornell Box"),
        //    shader: PathBuf::from("res/shaders/w06_cornell_box.wgsl"),
        //    model: Some(cornell_box_path.clone()),
        //    camera: cornell_box_camera.clone(),
        //    res: (512, 512),
        //},
        //SceneDescriptor {
        //    name: String::from("W7 Progressive Illum"),
        //    shader: PathBuf::from("res/shaders/w07_cornell_box.wgsl"),
        //    model: Some(cornell_box_with_blocks_path.clone()),
        //    camera: cornell_box_camera.clone(),
        //    res: (512, 512),
        //},
        //SceneDescriptor {
        //    name: String::from("W8 Fresnel"),
        //    shader: PathBuf::from("res/shaders/w08_1.wgsl"),
        //    model: Some(cornell_box_with_blocks_path.clone()),
        //    camera: cornell_box_camera.clone(),
        //    res: (512, 512),
        //},  
    ])
}