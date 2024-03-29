// Copyright (C) 2022 - 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

pub mod style;

use crate::{relationships::{Relationship, Relationships}, gui::Size};
use roxmltree as xml;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
pub struct DrawingObject {
    extent: Option<Extent>,
    graphic: GraphicObject,
}

impl DrawingObject {
    pub fn parse_inline_object(node: &xml::Node, relationships: &Relationships) -> Self {
        let mut object = DrawingObject {
            extent: None,
            graphic: GraphicObject::Empty
        };

        for child in node.children() {
            match child.tag_name().name() {
                "extent" => object.extent = Some(Extent::parse_xml(&child)),
                "graphic" => object.graphic = GraphicObject::parse_xml(&child, relationships),

                _ => ()
            }
        }

        object
    }

    pub fn size(&self) -> Size<f32> {
        match self.extent {
            Some(extent) => {
                // 20.1.2.1 EMU Unit of Measurement
                // 1 emu = 1/914400 inch
                Size::new(
                    extent.width as f32 * crate::word_processing::HALF_POINT * 72.0 / 914400.0,
                    extent.height as f32 * crate::word_processing::HALF_POINT * 72.0 / 914400.0,
                )
            }
            None => Default::default(),
        }
    }

    /*
    pub fn draw<'a>(&self, page: usize, position: Vector2f, painter: &mut crate::wp::painter::Painter) {
        match &self.graphic {
            GraphicObject::Empty => (),

            GraphicObject::Picture(picture) => {
                let image = picture.fill.as_ref().unwrap().blip.as_ref().unwrap().image.as_ref().unwrap();

                let mut texture = sfml::graphics::Texture::new().unwrap();
                texture.load_from_image(image, sfml::graphics::Rect::new(0, 0, image.size().x as i32, image.size().y as i32))
                    .expect("Failed to load image");

                let mut sprite = sfml::graphics::Sprite::new();
                sprite.set_texture(&texture, true);

                let rect = sprite.global_bounds();
                let size = self.size();
                sprite.set_scale((
                    size.width() / rect.width,
                    size.height() / rect.height
                ));

                sprite.set_position(position);
                let page = painter.get_page(page);
                page.as_ref().borrow_mut().draw(&sprite);
            }
        }
    }
    */
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// 20.4.2.7 extent (Drawing Object Size)
pub struct Extent {
    width: u32,
    height: u32,
}

impl Extent {
    pub fn parse_xml(node: &xml::Node) -> Self {
        Self {
            width: node.attribute("cx").unwrap().parse().unwrap(),
            height: node.attribute("cy").unwrap().parse().unwrap(),
        }
    }
}

#[derive(Debug)]
pub enum GraphicObject {
    Empty,

    Picture(Picture)
}

impl GraphicObject {
    pub fn parse_xml(node: &xml::Node, relationships: &Relationships) -> Self {
        for child in node.children() {
            if child.tag_name().name() == "graphicData" {
                for child in child.children() {
                    if child.tag_name().name() == "pic" {
                        return GraphicObject::Picture(Picture::parse_xml(&child, relationships));
                    }
                }
            }
        }

        panic!("Unknown GraphicData!");
    }
}

#[derive(Debug)]
pub struct Picture {
    fill: Option<PictureFill>,
}

impl Picture {
    pub fn parse_xml(node: &xml::Node, relationships: &Relationships) -> Self {
        let mut picture = Picture {
            fill: None
        };

        for child in node.children() {
            match child.tag_name().name() {
                "blipFill" => picture.fill = Some(PictureFill::parse_xml(&child, relationships)),

                _ => (),
            }
        }

        picture
    }
}

#[derive(Debug)]
pub struct PictureFill {
    blip: Option<Blip>
}

impl PictureFill {
    pub fn parse_xml(node: &xml::Node, relationships: &Relationships) -> Self {
        let mut fill = PictureFill {
            blip: None
        };

        for child in node.children() {
            match child.tag_name().name() {
                "blip" => fill.blip = Some(Blip::parse_xml(&child, relationships)),

                _ => (),
            }
        }

        fill
    }
}

pub struct Blip {
    embedded: Option<Rc<RefCell<Relationship>>>,
    // image: Option<sfml::graphics::Image>,
}

impl Blip {
    pub fn parse_xml(node: &xml::Node, relationships: &Relationships) -> Self {
        let mut blip = Blip {
            embedded: None,
            // image: None,
        };

        for attribute in node.attributes() {
            match attribute.name() {
                "embed" => {
                    let relationship = relationships.find(attribute.value()).expect("Failed to find embedded picture").clone();

                    let rela = relationship.as_ref().borrow();
                    assert_eq!(rela.relation_type, crate::relationships::RelationshipType::Image);

                    // blip.image = Some(sfml::graphics::Image::from_memory(&rela.data).expect("Failed to load image"));
                    drop(rela);

                    blip.embedded = Some(relationship);
                }
                _ => ()
            }
        }

        blip
    }
}

impl core::fmt::Debug for Blip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        /// To avoid dumping all the pixels, we use a fake object.
        #[derive(Debug)]
        struct Image {
            #[allow(dead_code)]
            width: u32,

            #[allow(dead_code)]
            height: u32
        }

        // let image = match &self.image {
        //     Some(image) => Some(Image {
        //         width: image.size().x,
        //         height: image.size().y,
        //     }),
        //     None => None,
        // };

        f.debug_struct("Blip")
            .field("embedded", &self.embedded)
            // .field("image", &image)
            .finish()
    }
}
