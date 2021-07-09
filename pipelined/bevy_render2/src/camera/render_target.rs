use bevy_window::WindowId;

enum RenderTarget {
    Window(WindowId),
    TextureView(TextureView),
}
