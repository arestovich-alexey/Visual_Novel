#![allow(unused_imports)]

//#[cfg(any(feature="texture_graphics",feature="simple_graphics",feature="text_graphics"))]
use crate::Colour;

#[cfg(feature="texture_graphics")]
use crate::image::{ImageBase,Texture};
#[cfg(feature="texture_graphics")]
use super::TextureGraphics;

#[cfg(feature="simple_graphics")]
use super::{SimpleGraphics,SimpleObject};

#[cfg(feature="text_graphics")]
use crate::text::Character;
#[cfg(feature="text_graphics")]
use super::TextGraphics;

use glium::{
    // enums
    DrawError,
    // traits
    Surface,
    // structs
    Frame,
    DrawParameters,
    Display,
    index::{
        PrimitiveType, // enum
        NoIndices,
    },
};

use core::ops::Range;

/// Настройки графических основ
pub struct GraphicsSettings{
    #[cfg(feature="texture_graphics")]
    pub texture_vertex_buffer_size:usize,
    #[cfg(feature="simple_graphics")]
    pub simple_vertex_buffer_size:usize,
    #[cfg(feature="text_graphics")]
    pub text_vertex_buffer_size:usize,
}

impl GraphicsSettings{
    pub const fn new()->GraphicsSettings{
        Self{
            #[cfg(feature="texture_graphics")]
            texture_vertex_buffer_size:8usize,
            #[cfg(feature="simple_graphics")]
            simple_vertex_buffer_size:100usize,
            #[cfg(feature="text_graphics")]
            text_vertex_buffer_size:2000usize,
        }
    }
}

/// Графические основы.
pub struct Graphics2D{
    #[cfg(feature="texture_graphics")]
    texture:TextureGraphics,
    #[cfg(feature="simple_graphics")]
    simple:SimpleGraphics,
    #[cfg(feature="text_graphics")]
    text:TextGraphics,
}

impl Graphics2D{
    pub fn new(window:&Display,settings:GraphicsSettings,glsl:u16)->Graphics2D{
        Self{
            #[cfg(feature="texture_graphics")]
            texture:TextureGraphics::new(window,settings.texture_vertex_buffer_size,glsl),
            #[cfg(feature="simple_graphics")]
            simple:SimpleGraphics::new(window,settings.simple_vertex_buffer_size,glsl),
            #[cfg(feature="text_graphics")]
            text:TextGraphics::new(window,settings.text_vertex_buffer_size,glsl),
        }
    }

    // Сохраняет координаты картинки в выбранной области в буфере,
    // чтобы постоянно не загружать заново при отрисовке
    // Используется только для невращающихся изображений
    // Для вывода изображения из этой области используется функция 'draw_range_image'
    // Возращает номер области, если она не выходит за границы буфера
    #[cfg(feature="texture_graphics")]
    pub fn bind_image(&mut self,range:Range<usize>,image_base:ImageBase)->Option<usize>{
        let data=image_base.vertex_buffer();
        self.texture.bind_range(range,&data)
    }

    // Сохраняет координаты картинки в выбранной области в буфере,
    // чтобы постоянно не загружать заново при отрисовке
    // Используется только для вращающихся изображений
    // Для вывода изображения из этой области используется функция 'draw_rotate_range_image'
    // Возращает номер области, если она не выходит за границы буфера
    #[cfg(feature="texture_graphics")]
    pub fn bind_rotating_image(&mut self,range:Range<usize>,image_base:ImageBase)->Option<usize>{
        let data=image_base.rotation_vertex_buffer();
        self.texture.bind_range(range,&data)
    }

    #[cfg(feature="texture_graphics")]
    pub fn rewrite_range_image(&mut self,range:usize,image_base:ImageBase)->Option<()>{
        let data=image_base.rotation_vertex_buffer();
        self.texture.rewrite_range(range,&data)
    }

    #[inline(always)]
    #[cfg(feature="texture_graphics")]
    pub fn unbind_texture(&mut self,index:usize){
        self.texture.unbind(index)
    }

    #[cfg(feature="texture_graphics")]
    pub fn draw_range_image(
        &self,
        index:usize,
        texture:&Texture,
        colour_filter:Colour,
        draw_parameters:&mut DrawParameters,
        frame:&mut Frame
    )->Result<(),DrawError>{
        let indices=NoIndices(PrimitiveType::TriangleStrip);
        self.texture.draw_range(
            index,
            texture,
            colour_filter,
            indices,
            draw_parameters,
            frame
        )
    }

    #[cfg(feature="texture_graphics")]
    pub fn draw_move_range_image(
        &self,
        index:usize,
        texture:&Texture,
        colour_filter:Colour,
        [dx,dy]:[f32;2],
        draw_parameters:&mut DrawParameters,
        frame:&mut Frame
    )->Result<(),DrawError>{
        let indices=NoIndices(PrimitiveType::TriangleStrip);
        self.texture.draw_move_range(
            index,
            texture,
            colour_filter,
            [dx,-dy],
            indices,
            draw_parameters,
            frame
        )
    }

    #[cfg(feature="texture_graphics")]
    pub fn draw_rotate_range_image(
        &self,
        index:usize,
        texture:&Texture,
        colour_filter:Colour,
        angle:f32,
        draw_parameters:&mut DrawParameters,
        frame:&mut Frame
    )->Result<(),DrawError>{
        let indices=NoIndices(PrimitiveType::TriangleStrip);
        self.texture.draw_rotate_range(
            index,
            texture,
            colour_filter,
            angle,
            indices,
            draw_parameters,
            frame
        )
    }
}

/// Простой интерфейс для связи кадра и графических функций
pub struct Graphics<'graphics,'frame>{
    graphics:&'graphics Graphics2D,
    frame:&'frame mut Frame,
}

impl<'graphics,'frame> Graphics<'graphics,'frame>{
    #[inline(always)]
    pub fn new(graphics:&'graphics Graphics2D,frame:&'frame mut Frame)->Graphics<'graphics,'frame>{
        Self{
            graphics,
            frame
        }
    }

    #[inline(always)]
    pub fn frame(&mut self)->&mut Frame{
        self.frame
    }

    #[inline(always)]
    pub fn clear_colour(&mut self,colour:[f32;4]){
        self.frame.clear_color(colour[0],colour[1],colour[2],colour[3]);
    }

    /// Рисует простой объект.
    #[inline(always)]
    #[cfg(feature="simple_graphics")]
    pub fn draw_simple<'a,O:SimpleObject<'a>>(
        &mut self,
        object:&O,
        draw_parameters:&mut DrawParameters
    )->Result<(),DrawError>{
        self.graphics.simple.draw(object,draw_parameters,self.frame)
    }

    /// Рисует и сдвигает простой объект.
    #[inline(always)] 
    #[cfg(feature="simple_graphics")]
    pub fn draw_move_simple<'a,O:SimpleObject<'a>>(
        &mut self,
        object:&O,
        movement:[f32;2],
        draw_parameters:&mut DrawParameters
    )->Result<(),DrawError>{
        self.graphics.simple.draw_move(object,movement,draw_parameters,self.frame)
    }

    /// Рисует один символ.
    #[inline(always)]
    #[cfg(feature="text_graphics")]
    pub fn draw_character(
        &mut self,
        colour:Colour,
        character:&Character,
        draw_parameters:&mut DrawParameters
    )->Result<(),DrawError>{
        self.graphics.text.draw_character(character,colour,draw_parameters,self.frame)
    }

    /// Рисует изображение на основе image_base.
    #[inline(always)] 
    #[cfg(feature="texture_graphics")]
    pub fn draw_image(
        &mut self,
        image_base:&ImageBase,
        texture:&Texture,
        draw_parameters:&mut DrawParameters
    )->Result<(),DrawError>{
        self.graphics.texture.draw_image(image_base,texture,draw_parameters,self.frame)
    }

    /// Рисует изображение на основе image_base c поворотом в 'angle' градусов
    #[inline(always)]
    #[cfg(feature="texture_graphics")]
    pub fn draw_rotate_image(
        &mut self,
        image_base:&ImageBase,
        texture:&Texture,
        angle:f32,
        draw_parameters:&mut DrawParameters
    )->Result<(),DrawError>{
        self.graphics.texture.draw_rotate_image(image_base,texture,angle,self.frame,draw_parameters)
    }
}

// Функции для работы с областями
impl<'graphics,'frame> Graphics<'graphics,'frame>{
    /// Рисует изображение на основе
    /// данных из области.
    #[inline(always)]
    #[cfg(feature="texture_graphics")]
    pub fn draw_range_image(
        &mut self,
        index:usize,
        texture:&Texture,
        colour_filter:Colour,
        draw_parameters:&mut DrawParameters
    )->Result<(),DrawError>{
        self.graphics.draw_range_image(
            index,
            texture,
            colour_filter,
            draw_parameters,
            self.frame,
        )
    }

    /// Рисует и сдвигает изображение на основе
    /// данных из области.
    #[inline(always)]
    #[cfg(feature="texture_graphics")]
    pub fn draw_move_range_image(
        &mut self,
        index:usize,
        texture:&Texture,
        colour_filter:Colour,
        movement:[f32;2],
        draw_parameters:&mut DrawParameters
    )->Result<(),DrawError>{
        self.graphics.draw_move_range_image(
            index,
            texture,
            colour_filter,
            movement,
            draw_parameters,
            &mut self.frame
        )
    }

    /// Рисует изображение с поворотом в 'angle' градусов на основе
    /// данных из области.
    #[inline(always)]
    #[cfg(feature="texture_graphics")]
    pub fn draw_rotate_range_image(
        &mut self,
        index:usize,
        texture:&Texture,
        colour_filter:Colour,
        angle:f32,
        draw_parameters:&mut DrawParameters
    )->Result<(),DrawError>{
        self.graphics.draw_rotate_range_image(
            index,
            texture,
            colour_filter,
            angle,
            draw_parameters,
            &mut self.frame
        )
    }
}