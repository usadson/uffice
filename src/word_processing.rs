// Copyright (C) 2022 - 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use roxmltree as xml;
use uffice_lib::namespaces::XMLNS_RELATIONSHIPS;
use unicode_segmentation::UnicodeSegmentation;

use std::{cell::RefCell, rc::Rc};

use crate::{
    *,
    text_settings::{
        PageSettings,
        Numbering,
        TextJustification,
    },
    error::Error,
    gui::{
        Position,
        Rect,
        Size,
    },
    relationships::Relationships,
    wp::{
        Document,
        layout::LineLayout,
        Node,
        numbering,
    },
    gui::painter::{
        TextCalculator,
        FontSpecification,
    },
    style::StyleManager,
};

const CORE_FACTOR: f32 = 1.0f32;
pub const TWELFTEENTH_POINT: f32 = 1f32 / 12.0 * CORE_FACTOR;
pub const HALF_POINT: f32 = 0.5 * CORE_FACTOR;
const LINE_SPACING: f32 = 6.0 * CORE_FACTOR;

struct Context<'a> {
    #[allow(dead_code)]
    document: &'a xml::Document<'a>,

    text_calculator: &'a mut dyn gui::painter::TextCalculator,
    progress_sender: &'a dyn Fn(f32),

    document_relationships: &'a Relationships,
    style_manager: &'a StyleManager,
    page_settings: PageSettings,

    numbering_manager: wp::numbering::NumberingManager,
}

fn load_page_settings(document: &xml::Document) -> Result<PageSettings, Error> {
    for root_child in document.root_element().first_child().unwrap().children() {
        // println!("Direct Root Child {}", root_child.tag_name().name());
        if root_child.tag_name().name() != "sectPr" {
            continue;
        }

        let mut page_size = Size::new(10, 10);
        let mut margins = Rect::<u32>::empty();

        let mut offset_header = 0;
        let mut offset_footer = 0;

        for child in root_child.children() {
            match child.tag_name().name() {
                "pgSz" => {
                    page_size = Size::new(
                        str::parse(child.attribute((WORD_PROCESSING_XML_NAMESPACE, "w")).expect("No width parameter"))?,
                        str::parse(child.attribute((WORD_PROCESSING_XML_NAMESPACE, "h")).expect("No height parameter"))?
                    );
                }
                "pgMar" => {
                    for attribute in child.attributes() {
                        match attribute.name() {
                            "left" => margins.left = str::parse(attribute.value())?,
                            "right" => margins.right = str::parse(attribute.value())?,
                            "top" => margins.top = str::parse(attribute.value())?,
                            "bottom" => margins.bottom = str::parse(attribute.value())?,
                            "header" => offset_header = str::parse(attribute.value())?,
                            "footer" => offset_footer = str::parse(attribute.value())?,
                            _ => ()
                        }
                    }
                }
                _ => ()
            }
        }

        return Ok(PageSettings::new(page_size, margins, offset_header, offset_footer));
    }

    panic!("No direct child \"sectPr\" of root element found :(");
}

pub type DocumentResult = Rc<RefCell<Node>>;

pub fn process_document(document: &xml::Document, style_manager: &StyleManager,
                        document_relationships: &Relationships,
                        numbering_manager: wp::numbering::NumberingManager,
                        document_properties: wp::document_properties::DocumentProperties,
                        text_calculator: &mut dyn gui::painter::TextCalculator,
                        progress_sender: &dyn Fn(f32)) -> DocumentResult {
    let text_settings = style_manager.default_text_settings();
    //text_settings.font = Some(String::from("Calibri"));

    let page_settings = load_page_settings(document).unwrap();

    let mut position = Position::new(
        page_settings.margins.left as f32 * TWELFTEENTH_POINT,
        page_settings.margins.top as f32 * TWELFTEENTH_POINT
    );

    // println!("Rendering Document:\n\tSize: {} x {}\n\tRender Size: {} x {}",
    //    page_settings.size.width,
    //    page_settings.size.height,
    //    page_settings.size.width as f32 / 12f32,
    //    page_settings.size.height as f32 / 12f32
    //);

    let doc = Rc::new(
        RefCell::new(
            Document::new(
                text_settings,
                page_settings,
                document_properties
            )
        )
    );

    let mut context = Context{
        document,
        text_calculator,
        progress_sender,

        document_relationships,
        style_manager,
        page_settings,

        numbering_manager,
    };

    for child in document.root_element().children() {
        // println!("{}", child.tag_name().name());

        if child.tag_name().name() == "body" {
            position = process_body_element(&mut context, &doc, &child, position);
        }
    }

    doc
}

fn process_drawing_element(context: &mut Context, parent: &Rc<RefCell<Node>>,
                           node: &xml::Node, position: Position<f32>) -> Position<f32> {
    for child in node.children() {
        match child.tag_name().name() {
            "inline" => {
                let drawing_object = drawing_ml::DrawingObject::parse_inline_object(&child, context.document_relationships);
                let size = drawing_object.size();

                let inline_drawing = wp::create_child(&parent, wp::NodeData::Drawing(drawing_object));
                inline_drawing.borrow_mut().size = size;

                let mut parent = parent.borrow_mut();
                assert_eq!(parent.size, gui::Size::new(0.0, 0.0));
                parent.size = size;
            }

            _ => ()
        }
    }

    position
}

fn process_body_element(context: &mut Context,
                        parent: &Rc<RefCell<Node>>,
                        node: &xml::Node,
                        position: Position<f32>) -> Position<f32> {
    let mut position = position;

    let child_count = node.children().count();
    let mut child_idx = 0;

    for child in node.children() {
        match child.tag_name().name() {
            "p" => position = process_paragraph_element(context, &parent, &child, position),
            "sdt" => position = process_structured_document_tag(context, &parent, &child, position),
            _ => ()
        }

        let progress = child_idx as f32 / child_count as f32;
        (context.progress_sender)(progress);
        child_idx += 1;
    }

    position
}

fn process_break_element(parent: &Rc<RefCell<Node>>, line_layout: &mut LineLayout, node: &xml::Node) {
    let break_type = wp::BreakType::from_string(node.attribute((WORD_PROCESSING_XML_NAMESPACE, "type")));
    match break_type {
        wp::BreakType::Page => {
            line_layout.reset();

            let next_page = parent.borrow_mut().page_last + 1;

            let child = wp::create_child(&parent, wp::NodeData::Break);
            let mut child = child.borrow_mut();
            child.page_first = next_page;
            child.position = line_layout.position_on_line;
            child.set_last_page_number(next_page);
        }
        _ => {
            println!("[WP] TODO: unknown break type: \"{:?}\"", break_type);
        }
    }
}

fn process_hyperlink_element(context: &mut Context,
                             parent: &Rc<RefCell<Node>>,
                             line_layout: &mut wp::layout::LineLayout,
                             node: &xml::Node,
                             mut position: Position<f32>) -> Position<f32> {
    let hyperlink_ref = wp::append_child(parent, wp::Node::new(wp::NodeData::Hyperlink(Default::default())));

    for child in node.children() {
        // Text Run
        if child.tag_name().name() == "r" {
            position = process_text_run_element(context, &hyperlink_ref, line_layout, &child, position);
        }
    }

    let mut hyperlink = hyperlink_ref.borrow_mut();
    if let Some(relationship_id) = node.attribute((XMLNS_RELATIONSHIPS, "id")) {
        if let Some(relationship) = context.document_relationships.find(relationship_id) {
            if let wp::NodeData::Hyperlink(hyperlink) = &mut hyperlink.data {
                hyperlink.relationship = Some(relationship.clone());
            }
        } else {
            // println!("[WARNING] <w:hyperlink> relationship not found: \"{}\" (out of {} relationship(s))",
            //    relationship_id, context.document_relationships.len());
        }
    } else {
        // println!("[WARNING] <w:hyperlink> doesn't have an r:id attribute!");
    }

    position
}

fn process_paragraph_element(context: &mut Context,
                             parent: &Rc<RefCell<Node>>,
                             node: &xml::Node,
                             original_position: Position<f32>) -> Position<f32> {
    let paragraph = wp::append_child(&parent, wp::Node::new(wp::NodeData::Paragraph(wp::Paragraph)));

    //position.x = context.page_settings.margins.left as f32 * TWELFTEENTH_POINT;
    let mut line_layout = wp::layout::LineLayout::new(&context.page_settings, original_position.y());

    paragraph.borrow_mut().position = line_layout.position_on_line;
    let mut position = line_layout.position_on_line;
    //paragraph.borrow_mut().position = position;

    if let Some(first_child) = node.first_child() {
        // Paragraph Properties section 17.3.1.26
        if first_child.tag_name().name() == "pPr" {
            // println!("│  ├─ {}", first_child.tag_name().name());
            process_paragraph_properties_element_for_paragraph(context, &paragraph, &first_child);
        }
    }

    assert!(paragraph.try_borrow_mut().is_ok());

    {
        let pref = paragraph.as_ref().borrow();
        if let Some(numbering) = pref.text_settings.numbering.clone() {
            drop(pref);
            let node = numbering.create_node(&paragraph, &mut line_layout, context.text_calculator);
            *position.x_mut() += node.as_ref().borrow().size.width();
            // println!("Numbering Width: {}", node.as_ref().borrow().size.x);



            pub const NUMBERING_INDENTATION: f32 = 700.0 * TWELFTEENTH_POINT;

            let text_settings = &paragraph.as_ref().borrow().text_settings;
            if text_settings.indentation_left.is_some() {
                *position.x_mut() = text_settings.indent_one(position.x(), true);
            } else {
                *position.x_mut() = (position.x() / NUMBERING_INDENTATION + 1.0).floor() * NUMBERING_INDENTATION;
            }
        }
    }



    for child in node.children() {
        // println!("│  ├─ {}", child.tag_name().name());

        match child.tag_name().name() {
            // 17.16.22 hyperlink (Hyperlink)
            "hyperlink" => {
                position = process_hyperlink_element(context, &paragraph, &mut line_layout, &child, position);
            }

            // Text Run
            "r" => {
                position = process_text_run_element(context, &paragraph, &mut line_layout, &child, position);
            }

            _ => ()
        }
    }

    let mut paragraph = paragraph.borrow_mut();

    // let font = context.font_manager.load_font(&paragraph.text_settings);
    // let text = paragraph.text_settings.create_text(&font);
    let family_name = match &paragraph.text_settings.font {
        None => "Calibri",
        Some(font) => font,
    };
    let font_spec = FontSpecification::new(
        &family_name,
        paragraph.text_settings.non_complex_text_size.unwrap() as f32 * HALF_POINT,
        paragraph.text_settings.font_weight(),
    );

    // The cursor is probably somewhere in the middle of the line.
    // We should put it at the next line.
    //
    // NOTE: This isn't line/paragraph spacing; see below.
    if position != original_position {
        //position.y += text.global_bounds().height;
        // TODO?
    }

    let line_spacing;
    if line_layout.line_height() > 0.0 {
        line_spacing = line_layout.line_height();
    } else {
        line_spacing = context.text_calculator.line_spacing(font_spec).unwrap() as f32 * HALF_POINT;
    }

    let paragraph_spacing = paragraph.text_settings.spacing_below_paragraph.unwrap_or(0.0);

    assert!(line_spacing >= 0.0);
    assert!(paragraph_spacing >= 0.0);

    // println!("│  ├─ Advancing {}  +  {}", line_spacing, paragraph_spacing);
    *position.y_mut() += line_spacing + paragraph_spacing;

    let diff = position - original_position;
    paragraph.size = diff.into();

    position
}

// pPr
pub fn process_paragraph_properties_element(numbering_manager: &numbering::NumberingManager, style_manager: &StyleManager,
                                            paragraph_text_settings: &mut text_settings::TextSettings, node: &xml::Node) {
    for property in node.children() {
        match property.tag_name().name() {
            "ind" => paragraph_text_settings.parse_element_ind(&property),

            // 17.3.1.13 jc (Paragraph Alignment)
            "jc" => {
                let val = property.attribute((WORD_PROCESSING_XML_NAMESPACE, "val"))
                        .expect("No w:val in a <w:jc> element!");
                match val {
                    "start" => paragraph_text_settings.justify = Some(TextJustification::Start),

                    "center" => paragraph_text_settings.justify = Some(TextJustification::Center),

                    // TODO I can't find the "right" value to be valid in the
                    // ECMA Specification, but Microsoft Word seams to be using
                    // this property anyway, so I inserted the quirk below.
                    "end" | "right" => paragraph_text_settings.justify = Some(TextJustification::End),
                    _ => {
                        // println!("│  │  │  ├─ E: Unknown Attribute Value");
                    }
                }
            }

            "numPr" => process_numbering_definition_instance_reference_property(numbering_manager, &property, paragraph_text_settings),

            // Paragraph Style
            "pStyle" => {
                let style_id = property.attribute((WORD_PROCESSING_XML_NAMESPACE, "val"))
                        .expect("No w:val in a <w:pStyle> element!");
                style_manager.apply_paragraph_style(style_id, paragraph_text_settings);
            }

            // Run Properties section 17.3.2.28
            "rPr" => {
                //apply_run_properties_for_paragraph_mark(&property, paragraph_text_settings);
            }

            "spacing" => {
                for attribute in property.attributes() {
                    // println!("│  │  │  ├─ Spacing Attribute: {} = {}", attribute.name(), attribute.value());
                    match attribute.name() {
                        "after" => {
                            paragraph_text_settings.spacing_below_paragraph = Some(str::parse(attribute.value())
                                    .expect("Failed to parse <w:spacing> 'after' attribute"));
                        }
                        _ => ()
                    }
                }
            }
            _ => ()
        }
    }
}

fn process_paragraph_properties_element_for_paragraph(context: &Context, paragraph: &Rc<RefCell<wp::Node>>, node: &xml::Node) {
    let mut paragraph = paragraph.borrow_mut();
    let paragraph_text_settings = &mut paragraph.text_settings;

    process_paragraph_properties_element(&context.numbering_manager, context.style_manager, paragraph_text_settings, node);
}

// 17.3.1.19 numPr (Numbering Definition Instance Reference)
fn process_numbering_definition_instance_reference_property(numbering_manager: &wp::numbering::NumberingManager, node: &xml::Node, text_settings: &mut text_settings::TextSettings) {
    let mut numbering = Numbering{
        definition: None,
        level: None,
    };

    for child in node.children() {
        match child.tag_name().name() {
            // 17.9.3 ilvl (Numbering Level Reference)
            "ilvl" => {
                numbering.level = Some(child.attribute((WORD_PROCESSING_XML_NAMESPACE, "val"))
                        .expect("No w:val attribute on <w:ilvl>!").parse().unwrap())
            }

            // 17.9.18 numId (Numbering Definition Instance Reference)
            "numId" => {
                let instance_id = child.attribute((WORD_PROCESSING_XML_NAMESPACE, "val"))
                    .expect("No w:val attribute on <w:numId>!").parse().unwrap();

                numbering.definition = numbering_manager.find_definition_instance(instance_id);
            }

            _ => ()
        }
    }

    assert_eq!(numbering.definition.is_some(), numbering.level.is_some(), "Both should be None or both be Some");

    text_settings.numbering = Some(numbering);
}

/// Process the <w:docPartObj> element
/// This element in a child of the <w:sdtPr> elemennt
fn process_sdt_built_in_doc_part(context: &mut Context, parent: &Rc<RefCell<Node>>, node: &xml::Node) {

    for child in node.children() {
        // println!("│  │  │  ├─ {}", child.tag_name().name());

        match child.tag_name().name() {
            "docPartGallery" => process_sdt_document_part_gallery_filter(context, parent, &child),
            _ => ()
        }
    }
}

/// Process the w:docPartGallery
fn process_sdt_document_part_gallery_filter(_context: &mut Context, _parent: &Rc<RefCell<Node>>, node: &xml::Node) {
    for _attr in node.attributes() {
        // println!("│  │  │  │  ├─ Attribute \"{}\" => \"{}\"   in namespace \"{}\"", attr.name(), attr.value(), attr.namespace().unwrap_or(""));
    }
}

/// Process the <w:sdtPr> element
fn process_std_properties(context: &mut Context, parent: &Rc<RefCell<Node>>, node: &xml::Node) {
    for child in node.children() {
        // println!("│  │  ├─ {}", child.tag_name().name());

        match child.tag_name().name() {
            "docPartObj" => process_sdt_built_in_doc_part(context, parent, &child),
            _ => ()
        }
    }
}

/// Process the <w:sdtEndPr> element
fn process_sdt_end_character_properties(_context: &mut Context, _parent: &Rc<RefCell<Node>>, node: &xml::Node) {
    for _child in node.children() {
        // println!("│  │  ├─ {}", child.tag_name().name());
    }
}

/// Process the <w:sdtContent> element
fn process_sdt_content(context: &mut Context, parent: &Rc<RefCell<Node>>, node: &xml::Node, original_position: Position<f32>) -> Position<f32> {
    let mut position = original_position;

    for child in node.children() {
        // println!("│  │  ├─ {}", child.tag_name().name());
        match child.tag_name().name() {
            "p" => position = process_paragraph_element(context, parent, &child, position),
            _ => ()
        }
    }

    position
}

/// Process the <w:sdt> element
/// 17.5.2 Structured Document Tags
fn process_structured_document_tag(context: &mut Context,
                                   parent: &Rc<RefCell<Node>>,
                                   node: &xml::Node,
                                   original_position: Position<f32>) -> Position<f32> {
    let mut position = original_position;

    let sdt = wp::append_child(&parent, wp::Node::new(wp::NodeData::StructuredDocumentTag(Default::default())));

    for child in node.children() {
        // println!("│  ├─ {}", child.tag_name().name());

        match child.tag_name().name() {
            "sdtContent" => position = process_sdt_content(context, &sdt, &child, original_position),
            "sdtEndPr" => process_sdt_end_character_properties(context, &sdt, &child),
            "sdtPr" => process_std_properties(context, &sdt, &child),
            _ => panic!("Illegal <w:sdt> child named: \"{}\" in namespace \"{}\"", child.tag_name().name(), child.tag_name().namespace().unwrap_or(""))
        }
    }

    position
}

/// Process the w:t element.
fn process_text_element(context: &mut Context,
                        parent: &Rc<RefCell<Node>>,
                        line_layout: &mut wp::layout::LineLayout,
                        node: &xml::Node,
                        position: Position<f32>) -> Position<f32> {
    let mut position = position;

    let instruction = (|| {
        let text_run_borrow = parent.as_ref().borrow();
        let parent = text_run_borrow.parent.upgrade().unwrap();
        let parent_borrow = parent.as_ref().borrow();

        for child in parent_borrow.children.as_ref().unwrap() {
            let child = child.as_ref().borrow();
            match &child.data {
                crate::wp::NodeData::TextRun(run) => if let Some(instruction) = &run.instruction {
                    return Some(instruction.clone());
                }
                _ => ()
            }
        }

        None
    })();

    if let Some(field) = instruction {
        return process_text_element_in_instructed_field(context, parent, line_layout, position, &field);
    }

    let text_node = wp::append_child(&parent, wp::Node::new(wp::NodeData::Text()));

    for child in node.children() {
        if child.node_type() == xml::NodeType::Text {
            let text_string = child.text().unwrap();
            // println!("│  │  │  ├─ Text: \"{}\"", text_string);
            position = process_text_element_text(&text_node, line_layout, context.text_calculator, text_string, position);
        }
    }

    position
}

fn process_text_element_in_instructed_field(context: &mut Context,
        parent: &Rc<RefCell<Node>>, line_layout: &mut LineLayout,
        _position: Position<f32>, field: &wp::instructions::Field) -> Position<f32> {
    append_text_element(&field.resolve_to_string(&parent), &parent, line_layout, context.text_calculator)
}

pub fn append_text_element(text_string: &str, parent: &Rc<RefCell<Node>>, line_layout: &mut wp::layout::LineLayout, text_calculator: &mut dyn TextCalculator) -> Position<f32> {
    let position = parent.as_ref().borrow().position;
    process_text_element_text(parent, line_layout, text_calculator, text_string, position)
}

pub fn process_text_element_text(parent: &Rc<RefCell<Node>>, line_layout: &mut wp::layout::LineLayout, text_calculator: &mut dyn TextCalculator, text_string: &str, original_position: Position<f32>) -> Position<f32> {
    #[derive(Debug)]
    enum LineStopReason {
        /// The end of the text was reached. This could also very well mean the
        /// whole string fitted on the text.
        EndReached,

        /// The line isn't the end of the text run, but this was all that could
        /// fit on the line.
        RestWasCutOff,
    }

    let mut position = original_position;

    let mut start_index = None;
    let mut previous_word_pair = None;

    let mut previous_stop_reason = None;

    let mut page_number = parent.as_ref().borrow().page_last;

    let text_settings = parent.borrow().text_settings.clone();
    let family_name = match &text_settings.font {
        None => "Calibri",
        Some(font) => font,
    };
    let font_spec = FontSpecification::new(
        &family_name, text_settings.non_complex_text_size.unwrap() as f32 * HALF_POINT, text_settings.font_weight(),
    );

    let line_spacing = text_calculator.line_spacing(font_spec).unwrap();

    let mut iter = UnicodeSegmentation::split_word_bound_indices(text_string).peekable();
    while let Some((index, word)) = iter.next() {
        let start;
        match start_index {
            Some(start_index) => start = start_index,
            None => {
                start_index = Some(index);
                start = index;
            }
        }

        let mut line = &text_string[start..(index + word.bytes().count())];
        let text_size = text_calculator.calculate_text_size(font_spec, line).unwrap();
        let mut width = text_size.width();

        let max_width_fitting_on_page = line_layout.page_horizontal_end - position.x();

        #[cfg(feature = "debug-text-layout")]
        println!("path \"{}\" x={} w={} max_on_page={} previous_stop={:?}", line, position.x(), width, max_width_fitting_on_page, previous_stop_reason);

        if max_width_fitting_on_page < 0.0 || previous_stop_reason.is_some() {
            *position.y_mut() += text_size.height() + line_spacing * LINE_SPACING;

            if position.y() > line_layout.page_vertical_end {
                page_number += 1;
                parent.borrow_mut().set_last_page_number(page_number);
                *position.y_mut() = line_layout.page_vertical_start;
                line_layout.reset();
            }

            *position.x_mut() = line_layout.page_horizontal_start;

            if iter.peek().is_some() {
                previous_stop_reason = None;
                continue;
            }
        }

        let stop_reason;

        #[cfg(feature = "debug-text-layout")]
        println!("width({}) < max_width_fitting_on_page({}) \"{}\"", width, max_width_fitting_on_page, line);

        if let Some((next_index, next_word)) = iter.peek() {
            let line_with_next = &text_string[start..(next_index + next_word.bytes().count())];

            let width_with_next = text_calculator.calculate_text_size(font_spec, line_with_next).unwrap().width();
            //text.set_string(line);

            if width < max_width_fitting_on_page && (iter.clone().skip(1).next().is_some() || width_with_next < max_width_fitting_on_page) {
                previous_word_pair = Some((index, word));
                continue;
            }

            stop_reason = LineStopReason::RestWasCutOff;
            start_index = None;

            if let Some((previous_word_index, previous_word)) = previous_word_pair {
                if !word.trim().is_empty() {
                    line = &text_string[start..(previous_word_index + previous_word.len())];
                    width = text_calculator.calculate_text_size(font_spec, line).unwrap().width();

                    start_index = Some(index);
                }
            }

            #[cfg(feature = "debug-text-layout")]
            println!("   stop_reason={:?} start_index={:?}", stop_reason, start_index);
        } else {
            stop_reason = LineStopReason::EndReached;
        }

        previous_word_pair = None;

        #[cfg(feature = "debug-text-layout")]
        {
            println!("│  │  │  │  ├─ Line: \"{}\", stop_reason={:?}", line, stop_reason);
            println!("│  │  │  │  ├─ Calculation: x={} w={} m={}", position.x, width, max_width_fitting_on_page);
        }

        let text_part_ref = wp::append_child(&parent, wp::Node::new(wp::NodeData::TextPart(wp::TextPart{ text: String::from(line) })));
        let mut text_part = text_part_ref.borrow_mut();
        text_part.page_first = page_number;
        text_part.page_last = page_number;
        text_part.size = text_calculator.calculate_text_size(font_spec, line).unwrap();

        text_part.position = match text_part.text_settings.justify.unwrap_or(TextJustification::Start) {
            TextJustification::Start => position,
            TextJustification::Center => Position::new(
                line_layout.page_horizontal_start + (line_layout.page_horizontal_end - line_layout.page_horizontal_start - width) / 2.0,
                position.y()
            ),
            TextJustification::End => Position::new(line_layout.page_horizontal_end - width, position.y())
        };

        line_layout.add_line_height_candidate(text_part.size.height());
        *line_layout.position_on_line.x_mut() += width;

        *position.x_mut() += width;

        previous_stop_reason = Some(stop_reason);
    }

    assert!(previous_word_pair.is_none());
    position
}

/// 17.3.2.25 r (Text Run)
/// This element specifies a run of content in the parent field, hyperlink,
/// custom XML element, structured document tag, smart tag, or paragraph.
fn process_text_run_element(context: &mut Context,
                            parent: &Rc<RefCell<Node>>,
                            line_layout: &mut wp::layout::LineLayout,
                            node: &xml::Node,
                            position: Position<f32>) -> Position<f32> {
    let mut position = position;

    let text_run = wp::append_child(parent, wp::Node::new(wp::NodeData::TextRun(Default::default())));

    for text_run_property in node.children() {
        match text_run_property.tag_name().name() {
            // 17.3.3.1 br (Break)
            "br" => {
                process_break_element(&text_run, line_layout, &text_run_property);

                // TODO: phase out the following code when the `position`
                //       parameters are fully replaced by the LineLayout
                //       system.
                if let Some(children) = &text_run.borrow().children {
                    if let Some(child) = children.last() {
                        position = child.borrow().position;
                    }
                }
                // End TODO
            }

            "drawing" => {
                position = process_drawing_element(context, &text_run, &text_run_property, position);
            }

            "instrText" => {
                if let crate::wp::NodeData::TextRun(run) = &mut text_run.borrow_mut().data {
                    for child in node.children() {
                        if let Some(text) = child.text() {
                            run.instruction = Some(crate::wp::instructions::Field::parse(text));
                            break;
                        }
                    }
                }
            }

            "rPr" =>  {
                text_run.borrow_mut().text_settings.apply_run_properties_element(context.style_manager, &text_run_property);
            }

            "t" => {
                position = process_text_element(context, &text_run, line_layout, &text_run_property, position);
            }

            _ => ()
        }
    }

    position
}
