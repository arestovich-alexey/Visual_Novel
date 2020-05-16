use glium::{
    implement_vertex,
    Program,
    VertexBuffer,
    Display,
    Frame,
    DrawParameters,
};

const Points_limit:usize=100; // Максимальное количество точек для одного объекта

implement_vertex!(Point2D,position);
#[derive(Copy,Clone)]
pub struct Point2D{
    pub position:[f32;2],
}

// Для простых одноцветных объектов,
// состоящих из менее, чем Points_limit точек
// 
pub struct SimpleGraphics{
    pub vertex_buffer:VertexBuffer<Point2D>,
    pub program:Program,
}

impl SimpleGraphics{
    pub fn new(display:&Display)->SimpleGraphics{
        let vertex_shader=r#"
            #version 140

            in vec2 position;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }
        "#;

        let fragment_shader=r#"
            #version 140

            out vec4 color;

            uniform vec4 colour;

            void main() {
                color = colour;
            }
        "#;

        Self{
            vertex_buffer:VertexBuffer::empty_dynamic(display,Points_limit).unwrap(),
            program:Program::from_source(display,vertex_shader,fragment_shader,None).unwrap(),
        }
    }

    #[inline(always)]
    pub fn draw<O:SimpleObject>(
        &self,
        object:&O,
        draw_parameters:&mut DrawParameters,
        frame:&mut Frame
    ){
        object.draw_simple(draw_parameters,frame,self)
    }
}

pub trait SimpleObject{
    fn draw_simple(&self,draw_parameters:&mut DrawParameters,frame:&mut Frame,graphics:&SimpleGraphics);
}