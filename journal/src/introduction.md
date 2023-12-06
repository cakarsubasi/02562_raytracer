# Introduction

This is the lab journal for the 02562 - Introduction to Rendering in DTU. Since I have went somewhat off the rails for the course, I have included this section to summarize the organization of code and have tried my best to document usage of third party code.

Special thanks to Jeppe Revall Frisvad (the course instructor) for guidance and many code samples (just about everywhere in the shaders, and also the bsp tree).

Thanks to A.B. Sørensen for providing code samples for the UI.

Thanks to Ben Hansen and his excellent Rust Wgpu tutorials and code samples which was used for parts of the render engine.

Code organization:
```
- res/  # resource folder
|- models/  # contains .obj and .mtl files
|- shaders/  # contains wgsl shaders
|- textures/  # contains texture files

- src/  # source folder
|- bindings/  # contains code handling bind groups and other
|  |          # render engine related stuff
|  |
|  |- mod.rs  # contains bind group interfaces
|  |- uniform.rs  # contains uniform variable bind group management
|  |- vertex.rs  # contains vertex buffer layout and constants
|  |             # (for the ray tracing canvas only)
|  |- texture.rs  # contains texture bind group management
|  |- mesh.rs  # contains vertex and index buffer layouts
|  |           # (for ray tracing canvas only)
|  |- storage_mesh.rs  # contains storage buffer management
|  |                   # for triangular meshes
|  |- bsp_tree.rs  # contains BSP tree storage buffer management
|                  # for triangular meshes
|
|- data_structures/  # data structure code that do not fit
|  |                 # anywhere else
|  |- vector.rs  # generic vector type
|  |- bbox.rs  # axis aligned bounding box type (adapted from Optix
|  |           # rendering framework in the course website)
|  |- bsp_tree.rs  # binary space partitioning tree type
|                  # (adapted from provided javascript code)
|
|- camera.rs  # camera management and camera controller code
|             # (partially adapted from Learn WGPU code)
|
|           # UI code, adapted from EGUI-WGPU template code
|           # From A.B. Sørensen
|- command.rs  # Commands from UI to rendering thread
|- control_panel.rs  # Control panel UI code
|- gpu_handles.rs  # Helper types for the control panel
|
|- mesh.rs  # mesh importing
|- render_state.rs  # the renderer, adapted partially from LearnWGPU
|
|- scenes.rs  # helper type for easy "scene" switching
|- tools.rs  # helper type for render statistics
|- lib.rs  # code for the UI thread and render thread
|- main.rs  # entry point
    
```
