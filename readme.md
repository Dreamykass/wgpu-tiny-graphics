# wgpu-tiny-graphics

Work-in-progress.

## Goal
A tiny example/exploration on how to write graphics.

Drawing is supposed to be renderer-based, where the renderers
hold data required to render specific kinds of things. 

Doing the actual graphics is supposed to look like so:
```rust
Event::RedrawRequested(_) 
=> match graphics_state.begin_current_frame() {
  ... // some error-handling here
  Ok(mut current_frame) => { // frame is ok for drawing
    vertex_renderer.clear(); // clear the internal buffer
    vertex_renderer.push(vertices); // push to it
    vertex_renderer.draw(&mut current_frame); // draw it

    current_frame.finish_and_present(); 
    // actually present the frame
  }
},
```

The actual renderers would naturally have all different interfaces.
A simple vertex-drawing renderer would just get vertices 
and that's pretty much it. But there could be gui renderers,
textured-quads renderers, glyph/text renderers, 
particle renderers, etc.

## 
Big thanks to <https://sotrh.github.io/learn-wgpu/> for the tutorial.
