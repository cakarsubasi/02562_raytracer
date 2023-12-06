use std::{path::PathBuf, sync::Arc};

use crate::camera::Camera;

#[derive(Default, Debug, Copy, Clone)]
pub enum VertexType {
    #[default]
    Split,
    Combined,
}

#[derive(Debug, Clone)]
pub struct SceneDescriptor {
    pub name: String,
    pub shader: PathBuf,
    pub vertex_type: VertexType,
    pub model: Option<PathBuf>,
    pub background_hdri: Option<PathBuf>,
    pub camera: Camera,
    pub res: (u32, u32),
}

impl Default for SceneDescriptor {
    fn default() -> Self {
        Self {
            name: Default::default(),
            shader: Default::default(),
            vertex_type: Default::default(),
            background_hdri: None,
            model: Default::default(),
            camera: Default::default(),
            res: (512, 512),
        }
    }
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

    let dragon_camera = Camera {
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
    let dragon_path = PathBuf::from("res/models/dragon.obj");

    let campus_background_path = PathBuf::from("res/textures/luxo_pxr_campus.jpg");
    let campus_background_hdr_path = PathBuf::from("res/textures/luxo_pxr_campus.hdr.png");

    Arc::new([
        SceneDescriptor {
            name: String::from("W1 E1"),
            shader: PathBuf::from("res/shaders/w1e1.wgsl"),
            model: None,
            camera: basic_scene_camera.clone(),
            res: (512, 512),
            ..Default::default()
        },
        SceneDescriptor {
            name: String::from("W1 E2"),
            shader: PathBuf::from("res/shaders/w1e2.wgsl"),
            model: None,
            camera: basic_scene_camera.clone(),
            res: (512, 512),
            ..Default::default()
        },
        SceneDescriptor {
            name: String::from("W1 E3"),
            shader: PathBuf::from("res/shaders/w1e3.wgsl"),
            model: None,
            camera: basic_scene_camera.clone(),
            res: (512, 512),
            ..Default::default()
        },
        SceneDescriptor {
            name: String::from("W1 E4"),
            shader: PathBuf::from("res/shaders/w1e4.wgsl"),
            model: None,
            camera: basic_scene_camera.clone(),
            res: (512, 512),
            ..Default::default()
        },
        SceneDescriptor {
            name: String::from("W1 E5"),
            shader: PathBuf::from("res/shaders/w1e5.wgsl"),
            model: None,
            camera: basic_scene_camera.clone(),
            res: (512, 512),
            ..Default::default()
        },
        SceneDescriptor {
            name: String::from("W1 E6"),
            shader: PathBuf::from("res/shaders/w1e6.wgsl"),
            model: None,
            camera: basic_scene_camera.clone(),
            res: (512, 512),
            ..Default::default()
        },
        SceneDescriptor {
            name: String::from("W2 E1"),
            shader: PathBuf::from("res/shaders/w2e1.wgsl"),
            model: None,
            camera: basic_scene_camera.clone(),
            res: (512, 512),
            ..Default::default()
        },
        SceneDescriptor {
            name: String::from("W2 E2"),
            shader: PathBuf::from("res/shaders/w2e2.wgsl"),
            model: None,
            camera: basic_scene_camera.clone(),
            res: (512, 512),
            ..Default::default()
        },
        SceneDescriptor {
            name: String::from("W2 E3"),
            shader: PathBuf::from("res/shaders/w2e3.wgsl"),
            model: None,
            camera: basic_scene_camera.clone(),
            res: (512, 512),
            ..Default::default()
        },
        SceneDescriptor {
            name: String::from("W2 E4"),
            shader: PathBuf::from("res/shaders/w2e4.wgsl"),
            model: None,
            camera: basic_scene_camera.clone(),
            res: (512, 512),
            ..Default::default()
        },
        SceneDescriptor {
            name: String::from("W2 E5"),
            shader: PathBuf::from("res/shaders/w2e5.wgsl"),
            model: None,
            camera: basic_scene_camera.clone(),
            res: (512, 512),
            ..Default::default()
        },
        SceneDescriptor {
            name: String::from("W3 E1"),
            shader: PathBuf::from("res/shaders/w3e1.wgsl"),
            model: None,
            camera: basic_scene_camera.clone(),
            res: (512, 512),
            ..Default::default()
        },
        SceneDescriptor {
            name: String::from("W3 E2"),
            shader: PathBuf::from("res/shaders/w3e2.wgsl"),
            model: None,
            camera: basic_scene_camera.clone(),
            res: (512, 512),
            ..Default::default()
        },
        SceneDescriptor {
            name: String::from("W3 E3"),
            shader: PathBuf::from("res/shaders/w3e3.wgsl"),
            model: None,
            camera: basic_scene_camera.clone(),
            res: (512, 512),
            ..Default::default()
        },
        SceneDescriptor {
            name: String::from("W3 E4"),
            shader: PathBuf::from("res/shaders/w3e4.wgsl"),
            model: None,
            camera: basic_scene_camera.clone(),
            res: (512, 512),
            ..Default::default()
        },
        SceneDescriptor {
            name: String::from("W5 E2 Teapot"),
            shader: PathBuf::from("res/shaders/w5e2.wgsl"),
            model: Some(teapot_path.clone()),
            camera: utah_teapot_camera.clone(),
            res: (800, 450),
            ..Default::default()
        },
        SceneDescriptor {
            name: String::from("W5 E3 Teapot"),
            shader: PathBuf::from("res/shaders/w5e3.wgsl"),
            model: Some(teapot_path.clone()),
            camera: utah_teapot_camera.clone(),
            res: (800, 450),
            ..Default::default()
        },
        SceneDescriptor {
            name: String::from("W5 E4 Cornell Box"),
            shader: PathBuf::from("res/shaders/w5e4.wgsl"),
            model: Some(cornell_box_with_blocks_path.clone()),
            camera: cornell_box_camera.clone(),
            res: (512, 512),
            ..Default::default()
        },
        SceneDescriptor {
            name: String::from("W5 E5 Cornell Box"),
            shader: PathBuf::from("res/shaders/w5e5.wgsl"),
            model: Some(cornell_box_with_blocks_path.clone()),
            camera: cornell_box_camera.clone(),
            res: (512, 512),
            ..Default::default()
        },
        SceneDescriptor {
            name: String::from("W6 E1 Teapot"),
            shader: PathBuf::from("res/shaders/w6e1.wgsl"),
            model: Some(teapot_path.clone()),
            camera: utah_teapot_camera.clone(),
            res: (800, 450),
            ..Default::default()
        },
        SceneDescriptor {
            name: String::from("W6 E1 Bunny"),
            shader: PathBuf::from("res/shaders/w6e1.wgsl"),
            model: Some(bunny_path.clone()),
            camera: bunny_camera.clone(),
            res: (512, 512),
            ..Default::default()
        },
        SceneDescriptor {
            name: String::from("W6 E1 Dragon"),
            shader: PathBuf::from("res/shaders/w6e1.wgsl"),
            model: Some(dragon_path.clone()),
            camera: dragon_camera.clone(),
            res: (800, 450),
            ..Default::default()
        },
        SceneDescriptor {
            name: String::from("W6 E2 Cornell Box"),
            shader: PathBuf::from("res/shaders/w6e2.wgsl"),
            model: Some(cornell_box_with_blocks_path.clone()),
            camera: cornell_box_camera.clone(),
            res: (512, 512),
            vertex_type: VertexType::Combined,
            ..Default::default()
        },
        SceneDescriptor {
            name: String::from("W6 E3 Cornell Box"),
            shader: PathBuf::from("res/shaders/w6e3.wgsl"),
            model: Some(cornell_box_path.clone()),
            camera: cornell_box_camera.clone(),
            res: (512, 512),
            vertex_type: VertexType::Combined,
            ..Default::default()
        },
        SceneDescriptor {
            name: String::from("W7 E1 Cornell Box"),
            shader: PathBuf::from("res/shaders/w7e1.wgsl"),
            model: Some(cornell_box_with_blocks_path.clone()),
            camera: cornell_box_camera.clone(),
            res: (512, 512),
            vertex_type: VertexType::Combined,
            ..Default::default()
        },
        SceneDescriptor {
            name: String::from("W7 E2 Cornell Box"),
            shader: PathBuf::from("res/shaders/w7e2.wgsl"),
            model: Some(cornell_box_with_blocks_path.clone()),
            camera: cornell_box_camera.clone(),
            res: (512, 512),
            vertex_type: VertexType::Combined,
            ..Default::default()
        },
        SceneDescriptor {
            name: String::from("W7 E3 Cornell Box"),
            shader: PathBuf::from("res/shaders/w7e3.wgsl"),
            model: Some(cornell_box_with_blocks_path.clone()),
            camera: cornell_box_camera.clone(),
            res: (512, 512),
            vertex_type: VertexType::Combined,
            ..Default::default()
        },
        SceneDescriptor {
            name: String::from("W8 E1 Cornell Box Balls"),
            shader: PathBuf::from("res/shaders/w8e1.wgsl"),
            model: Some(cornell_box_path.clone()),
            camera: cornell_box_camera.clone(),
            res: (512, 512),
            vertex_type: VertexType::Combined,
            ..Default::default()
        },
        SceneDescriptor {
            name: String::from("W8 E2 Cornell Box Balls"),
            shader: PathBuf::from("res/shaders/w8e2.wgsl"),
            model: Some(cornell_box_path.clone()),
            camera: cornell_box_camera.clone(),
            res: (512, 512),
            vertex_type: VertexType::Combined,
            ..Default::default()
        },
        SceneDescriptor {
            name: String::from("W8 E3 Absorption"),
            shader: PathBuf::from("res/shaders/w8e3.wgsl"),
            model: Some(cornell_box_path.clone()),
            camera: cornell_box_camera.clone(),
            res: (512, 512),
            vertex_type: VertexType::Combined,
            ..Default::default()
        },
        SceneDescriptor {
            name: String::from("W9 E1 Teapot"),
            shader: PathBuf::from("res/shaders/w9e1.wgsl"),
            model: Some(teapot_path.clone()),
            camera: utah_teapot_camera.clone(),
            res: (800, 450),
            vertex_type: VertexType::Combined,
            background_hdri: Some(campus_background_path.clone()),
        },
        SceneDescriptor {
            name: String::from("W9 E1 Bunny"),
            shader: PathBuf::from("res/shaders/w9e1.wgsl"),
            model: Some(bunny_path.clone()),
            camera: bunny_camera.clone(),
            res: (512, 512),
            vertex_type: VertexType::Combined,
            background_hdri: Some(campus_background_path.clone()),
        },
        SceneDescriptor {
            name: String::from("W9 E2 Teapot"),
            shader: PathBuf::from("res/shaders/w9e2.wgsl"),
            model: Some(teapot_path.clone()),
            camera: utah_teapot_camera.clone(),
            res: (800, 450),
            vertex_type: VertexType::Combined,
            background_hdri: Some(campus_background_hdr_path.clone()),
        },
        SceneDescriptor {
            name: String::from("W9 E2 Bunny"),
            shader: PathBuf::from("res/shaders/w9e2.wgsl"),
            model: Some(bunny_path.clone()),
            camera: bunny_camera.clone(),
            res: (512, 512),
            vertex_type: VertexType::Combined,
            background_hdri: Some(campus_background_hdr_path.clone()),
        },
        SceneDescriptor {
            name: String::from("W9 E3 Teapot"),
            shader: PathBuf::from("res/shaders/w9e3.wgsl"),
            model: Some(teapot_path.clone()),
            camera: utah_teapot_camera.clone(),
            res: (800, 450),
            vertex_type: VertexType::Combined,
            background_hdri: Some(campus_background_path.clone()),
        },
    ])
}
