Window - winit window
Surface - wgpu definition of window that we draw to. Is configured with a config(width, height, vsync, etc) and a Device
Instance - Handle to GPU
Adapter - Handle to GPU + settings, like power preference and compatibility.

Device - Requested from the adapter. Open connection to a graphics device.
Queue - Executes recorded CommandBuffer objects

CommandEncoder -
RenderPass - In-progress recording of a render pass. (Begin with clear color wipe, set pipeline, draw)

Pipeline - Describes the shaders that will be run, and the layout of the data that will be passed to them.

Texture - Handle to a texture on the GPU
TextureView - Describes a texture and associated metadata needed by a RenderPipeline or BindGroup
