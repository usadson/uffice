// Copyright (C) 2022 - 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use roxmltree as xml;

use uffice_lib::{
    EighteenthPoint,
    TwelfteenthPoint,
    WholePoint,
};

use std::{
    collections::HashMap,
    str::FromStr,
    num::ParseIntError,
};

use crate::{
    error::Error,
    WORD_PROCESSING_XML_NAMESPACE,
    text_settings::TextSettings, wp::table::TableProperties, serialize::FromXmlStandalone,
};

pub type ThemeSettings = crate::drawing_ml::style::StyleSettings;

/// ST_Border
#[derive(Copy, Clone, Debug, Default)]
pub enum BorderType {
    Nil,
    None,

    #[default]
    Single,
    Thick,
    Double,
    Dotted,
    Dashed,
    DotDash,
    DotDotDash,
    Triple,
    ThinThickSmallGap,
    ThickThinSmallGap,
    ThinThickThinSmallGap,
    ThinThickMediumGap,
    ThickThinMediumGap,
    ThinThickThinMediumGap,
    ThinThickLargeGap,
    ThickThinLargeGap,
    ThinThickThinLargeGap,
    Wave,
    DoubleWave,
    DashSmallGap,
    DashDotStroked,
    ThreeDEmboss,
    ThreeDEngrave,
    Outset,
    Inset,
    Apples,
    ArchedScallops,
    BabyPacifier,
    BabyRattle,
    Balloons3Colors,
    BalloonsHotAir,
    BasicBlackDashes,
    BasicBlackDots,
    BasicBlackSquares,
    BasicThinLines,
    BasicWhiteDashes,
    BasicWhiteDots,
    BasicWhiteSquares,
    BasicWideInline,
    BasicWideMidline,
    BasicWideOutline,
    Bats,
    Birds,
    BirdsFlight,
    Cabins,
    CakeSlice,
    CandyCorn,
    CelticKnotwork,
    CertificateBanner,
    ChainLink,
    ChampagneBottle,
    CheckedBarBlack,
    CheckedBarColor,
    Checkered,
    ChristmasTree,
    CirclesLines,
    CirclesRectangles,
    ClassicalWave,
    Clocks,
    Compass,
    Confetti,
    ConfettiGrays,
    ConfettiOutline,
    ConfettiStreamers,
    ConfettiWhite,
    CornerTriangles,
    CouponCutoutDashes,
    CouponCutoutDots,
    CrazyMaze,
    CreaturesButterfly,
    CreaturesFish,
    CreaturesInsects,
    CreaturesLadyBug,
    CrossStitch,
    Cup,
    DecoArch,
    DecoArchColor,
    DecoBlocks,
    DiamondsGray,
    DoubleD,
    DoubleDiamonds,
    Earth1,
    Earth2,
    EasterEggBasket,
    EclipsingSquares1,
    EclipsingSquares2,
    Eggplant,
    Fans,
    Film,
    Firecrackers,
    FlowersBlockPrint,
    FlowersDaisies,
    FlowersModern1,
    FlowersModern2,
    FlowersPansy,
    FlowersRedRose,
    FlowersRoses,
    FlowersTeacup,
    FlowersTiny,
    Gems,
    GingerbreadMan,
    Gradient,
    Handmade1,
    Handmade2,
    HeartBalloon,
    HeartGray,
    Hearts,
    HeebieJeebies,
    Holly,
    HouseFunky,
    Hypnotic,
    IceCreamCones,
    LightBulb,
    Lightning1,
    Lightning2,
    MapPins,
    MapleLeaf,
    MapleMuffins,
    Marquee,
    MarqueeToothed,
    Moons,
    Mosaic,
    MusicNotes,
    Northwest,
    Ovals,
    Packages,
    PalmsBlack,
    PalmsColor,
    PaperClips,
    PartyFavor,
    PartyGlass,
    Pencils,
    People,
    PeopleHats,
    PeopleWaving,
    Poinsettias,
    PostageStamp,
    Pumpkin1,
    PushPinNote1,
    PushPinNote2,
    Pyramids,
    PyramidsAbove,
    Quadrants,
    Rings,
    Safari,
    Sawtooth,
    SawtoothGray,
    ScaredCat,
    Seattle,
    ShadowedSquares,
    SharksTeeth,
    ShorebirdTracks,
    Skyrocket,
    SnowflakeFancy,
    Snowflakes,
    Sombrero,
    Southwest,
    Stars,
    Stars3d,
    StarsBlack,
    StarsShadowed,
    StarsTop,
    Sun,
    Swirligig,
    TornPaper,
    TornPaperBlack,
    Trees,
    TriangleParty,
    Triangles,
    Tribal1,
    Tribal2,
    Tribal3,
    Tribal4,
    Tribal5,
    Tribal6,
    TwistedLines1,
    TwistedLines2,
    Vine,
    Waveline,
    WeavingAngles,
    WeavingBraid,
    WeavingRibbon,
    WeavingStrips,
    WhiteFlowers,
    Woodwork,
    XIllusions,
    ZanyTriangles,
    ZigZag,
    ZigZagStitch,
    Custom,
}

#[derive(Clone, Debug)]
pub enum BorderTypeParseError {
    UnknownBorderType(String),
}

impl FromStr for BorderType {
    type Err = BorderTypeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "nil" => Ok(Self::Nil),
            "none" => Ok(Self::None),
            "single" => Ok(Self::Single),
            "thick" => Ok(Self::Thick),
            "double" => Ok(Self::Double),
            "dotted" => Ok(Self::Dotted),
            "dashed" => Ok(Self::Dashed),
            "dotDash" => Ok(Self::DotDash),
            "dotDotDash" => Ok(Self::DotDotDash),
            "triple" => Ok(Self::Triple),
            "thinThickSmallGap" => Ok(Self::ThinThickSmallGap),
            "thickThinSmallGap" => Ok(Self::ThickThinSmallGap),
            "thinThickThinSmallGap" => Ok(Self::ThinThickThinSmallGap),
            "thinThickMediumGap" => Ok(Self::ThinThickMediumGap),
            "thickThinMediumGap" => Ok(Self::ThickThinMediumGap),
            "thinThickThinMediumGap" => Ok(Self::ThinThickThinMediumGap),
            "thinThickLargeGap" => Ok(Self::ThinThickLargeGap),
            "thickThinLargeGap" => Ok(Self::ThickThinLargeGap),
            "thinThickThinLargeGap" => Ok(Self::ThinThickThinLargeGap),
            "wave" => Ok(Self::Wave),
            "doubleWave" => Ok(Self::DoubleWave),
            "dashSmallGap" => Ok(Self::DashSmallGap),
            "dashDotStroked" => Ok(Self::DashDotStroked),
            "threeDEmboss" => Ok(Self::ThreeDEmboss),
            "threeDEngrave" => Ok(Self::ThreeDEngrave),
            "outset" => Ok(Self::Outset),
            "inset" => Ok(Self::Inset),
            "apples" => Ok(Self::Apples),
            "archedScallops" => Ok(Self::ArchedScallops),
            "babyPacifier" => Ok(Self::BabyPacifier),
            "babyRattle" => Ok(Self::BabyRattle),
            "balloons3Colors" => Ok(Self::Balloons3Colors),
            "balloonsHotAir" => Ok(Self::BalloonsHotAir),
            "basicBlackDashes" => Ok(Self::BasicBlackDashes),
            "basicBlackDots" => Ok(Self::BasicBlackDots),
            "basicBlackSquares" => Ok(Self::BasicBlackSquares),
            "basicThinLines" => Ok(Self::BasicThinLines),
            "basicWhiteDashes" => Ok(Self::BasicWhiteDashes),
            "basicWhiteDots" => Ok(Self::BasicWhiteDots),
            "basicWhiteSquares" => Ok(Self::BasicWhiteSquares),
            "basicWideInline" => Ok(Self::BasicWideInline),
            "basicWideMidline" => Ok(Self::BasicWideMidline),
            "basicWideOutline" => Ok(Self::BasicWideOutline),
            "bats" => Ok(Self::Bats),
            "birds" => Ok(Self::Birds),
            "birdsFlight" => Ok(Self::BirdsFlight),
            "cabins" => Ok(Self::Cabins),
            "cakeSlice" => Ok(Self::CakeSlice),
            "candyCorn" => Ok(Self::CandyCorn),
            "celticKnotwork" => Ok(Self::CelticKnotwork),
            "certificateBanner" => Ok(Self::CertificateBanner),
            "chainLink" => Ok(Self::ChainLink),
            "champagneBottle" => Ok(Self::ChampagneBottle),
            "checkedBarBlack" => Ok(Self::CheckedBarBlack),
            "checkedBarColor" => Ok(Self::CheckedBarColor),
            "checkered" => Ok(Self::Checkered),
            "christmasTree" => Ok(Self::ChristmasTree),
            "circlesLines" => Ok(Self::CirclesLines),
            "circlesRectangles" => Ok(Self::CirclesRectangles),
            "classicalWave" => Ok(Self::ClassicalWave),
            "clocks" => Ok(Self::Clocks),
            "compass" => Ok(Self::Compass),
            "confetti" => Ok(Self::Confetti),
            "confettiGrays" => Ok(Self::ConfettiGrays),
            "confettiOutline" => Ok(Self::ConfettiOutline),
            "confettiStreamers" => Ok(Self::ConfettiStreamers),
            "confettiWhite" => Ok(Self::ConfettiWhite),
            "cornerTriangles" => Ok(Self::CornerTriangles),
            "couponCutoutDashes" => Ok(Self::CouponCutoutDashes),
            "couponCutoutDots" => Ok(Self::CouponCutoutDots),
            "crazyMaze" => Ok(Self::CrazyMaze),
            "creaturesButterfly" => Ok(Self::CreaturesButterfly),
            "creaturesFish" => Ok(Self::CreaturesFish),
            "creaturesInsects" => Ok(Self::CreaturesInsects),
            "creaturesLadyBug" => Ok(Self::CreaturesLadyBug),
            "crossStitch" => Ok(Self::CrossStitch),
            "cup" => Ok(Self::Cup),
            "decoArch" => Ok(Self::DecoArch),
            "decoArchColor" => Ok(Self::DecoArchColor),
            "decoBlocks" => Ok(Self::DecoBlocks),
            "diamondsGray" => Ok(Self::DiamondsGray),
            "doubleD" => Ok(Self::DoubleD),
            "doubleDiamonds" => Ok(Self::DoubleDiamonds),
            "earth1" => Ok(Self::Earth1),
            "earth2" => Ok(Self::Earth2),
            "easterEggBasket" => Ok(Self::EasterEggBasket),
            "eclipsingSquares1" => Ok(Self::EclipsingSquares1),
            "eclipsingSquares2" => Ok(Self::EclipsingSquares2),
            "eggplant" => Ok(Self::Eggplant),
            "fans" => Ok(Self::Fans),
            "film" => Ok(Self::Film),
            "firecrackers" => Ok(Self::Firecrackers),
            "flowersBlockPrint" => Ok(Self::FlowersBlockPrint),
            "flowersDaisies" => Ok(Self::FlowersDaisies),
            "flowersModern1" => Ok(Self::FlowersModern1),
            "flowersModern2" => Ok(Self::FlowersModern2),
            "flowersPansy" => Ok(Self::FlowersPansy),
            "flowersRedRose" => Ok(Self::FlowersRedRose),
            "flowersRoses" => Ok(Self::FlowersRoses),
            "flowersTeacup" => Ok(Self::FlowersTeacup),
            "flowersTiny" => Ok(Self::FlowersTiny),
            "gems" => Ok(Self::Gems),
            "gingerbreadMan" => Ok(Self::GingerbreadMan),
            "gradient" => Ok(Self::Gradient),
            "handmade1" => Ok(Self::Handmade1),
            "handmade2" => Ok(Self::Handmade2),
            "heartBalloon" => Ok(Self::HeartBalloon),
            "heartGray" => Ok(Self::HeartGray),
            "hearts" => Ok(Self::Hearts),
            "heebieJeebies" => Ok(Self::HeebieJeebies),
            "holly" => Ok(Self::Holly),
            "houseFunky" => Ok(Self::HouseFunky),
            "hypnotic" => Ok(Self::Hypnotic),
            "iceCreamCones" => Ok(Self::IceCreamCones),
            "lightBulb" => Ok(Self::LightBulb),
            "lightning1" => Ok(Self::Lightning1),
            "lightning2" => Ok(Self::Lightning2),
            "mapPins" => Ok(Self::MapPins),
            "mapleLeaf" => Ok(Self::MapleLeaf),
            "mapleMuffins" => Ok(Self::MapleMuffins),
            "marquee" => Ok(Self::Marquee),
            "marqueeToothed" => Ok(Self::MarqueeToothed),
            "moons" => Ok(Self::Moons),
            "mosaic" => Ok(Self::Mosaic),
            "musicNotes" => Ok(Self::MusicNotes),
            "northwest" => Ok(Self::Northwest),
            "ovals" => Ok(Self::Ovals),
            "packages" => Ok(Self::Packages),
            "palmsBlack" => Ok(Self::PalmsBlack),
            "palmsColor" => Ok(Self::PalmsColor),
            "paperClips" => Ok(Self::PaperClips),
            "partyFavor" => Ok(Self::PartyFavor),
            "partyGlass" => Ok(Self::PartyGlass),
            "pencils" => Ok(Self::Pencils),
            "people" => Ok(Self::People),
            "peopleHats" => Ok(Self::PeopleHats),
            "peopleWaving" => Ok(Self::PeopleWaving),
            "poinsettias" => Ok(Self::Poinsettias),
            "postageStamp" => Ok(Self::PostageStamp),
            "pumpkin1" => Ok(Self::Pumpkin1),
            "pushPinNote1" => Ok(Self::PushPinNote1),
            "pushPinNote2" => Ok(Self::PushPinNote2),
            "pyramids" => Ok(Self::Pyramids),
            "pyramidsAbove" => Ok(Self::PyramidsAbove),
            "quadrants" => Ok(Self::Quadrants),
            "rings" => Ok(Self::Rings),
            "safari" => Ok(Self::Safari),
            "sawtooth" => Ok(Self::Sawtooth),
            "sawtoothGray" => Ok(Self::SawtoothGray),
            "scaredCat" => Ok(Self::ScaredCat),
            "seattle" => Ok(Self::Seattle),
            "shadowedSquares" => Ok(Self::ShadowedSquares),
            "sharksTeeth" => Ok(Self::SharksTeeth),
            "shorebirdTracks" => Ok(Self::ShorebirdTracks),
            "skyrocket" => Ok(Self::Skyrocket),
            "snowflakeFancy" => Ok(Self::SnowflakeFancy),
            "snowflakes" => Ok(Self::Snowflakes),
            "sombrero" => Ok(Self::Sombrero),
            "southwest" => Ok(Self::Southwest),
            "stars" => Ok(Self::Stars),
            "stars3d" => Ok(Self::Stars3d),
            "starsBlack" => Ok(Self::StarsBlack),
            "starsShadowed" => Ok(Self::StarsShadowed),
            "starsTop" => Ok(Self::StarsTop),
            "sun" => Ok(Self::Sun),
            "swirligig" => Ok(Self::Swirligig),
            "tornPaper" => Ok(Self::TornPaper),
            "tornPaperBlack" => Ok(Self::TornPaperBlack),
            "trees" => Ok(Self::Trees),
            "triangleParty" => Ok(Self::TriangleParty),
            "triangles" => Ok(Self::Triangles),
            "tribal1" => Ok(Self::Tribal1),
            "tribal2" => Ok(Self::Tribal2),
            "tribal3" => Ok(Self::Tribal3),
            "tribal4" => Ok(Self::Tribal4),
            "tribal5" => Ok(Self::Tribal5),
            "tribal6" => Ok(Self::Tribal6),
            "twistedLines1" => Ok(Self::TwistedLines1),
            "twistedLines2" => Ok(Self::TwistedLines2),
            "vine" => Ok(Self::Vine),
            "waveline" => Ok(Self::Waveline),
            "weavingAngles" => Ok(Self::WeavingAngles),
            "weavingBraid" => Ok(Self::WeavingBraid),
            "weavingRibbon" => Ok(Self::WeavingRibbon),
            "weavingStrips" => Ok(Self::WeavingStrips),
            "whiteFlowers" => Ok(Self::WhiteFlowers),
            "woodwork" => Ok(Self::Woodwork),
            "xIllusions" => Ok(Self::XIllusions),
            "zanyTriangles" => Ok(Self::ZanyTriangles),
            "zigZag" => Ok(Self::ZigZag),
            "zigZagStitch" => Ok(Self::ZigZagStitch),
            "custom" => Ok(Self::Custom),
            _ => Err(BorderTypeParseError::UnknownBorderType(s.to_string()))
        }
    }
}

/// The properties of a border.
#[derive(Copy, Clone, Debug, Default)]
pub struct BorderProperties {
    pub border_type: BorderType,
    pub width: EighteenthPoint<u32>,
    pub spacing: WholePoint<u32>,
    pub color: HexColor,
}

#[derive(Debug)]
pub enum BorderPropertiesParseError {
    BorderTypeParseError(BorderTypeParseError),
    HexColorParseError(HexColorParseError),
    ParseIntError(ParseIntError),

    /// The `w:val` attribute is required by CT_Border.
    ValAttributeMissing,
}

impl From<BorderTypeParseError> for BorderPropertiesParseError {
    fn from(error: BorderTypeParseError) -> Self {
        BorderPropertiesParseError::BorderTypeParseError(error)
    }
}

impl From<HexColorParseError> for BorderPropertiesParseError {
    fn from(error: HexColorParseError) -> Self {
        BorderPropertiesParseError::HexColorParseError(error)
    }
}

impl From<ParseIntError> for BorderPropertiesParseError {
    fn from(error: ParseIntError) -> Self {
        BorderPropertiesParseError::ParseIntError(error)
    }
}

impl FromXmlStandalone for BorderProperties {
    type ParseError = BorderPropertiesParseError;

    fn from_xml(node: &xml::Node) -> Result<Self, BorderPropertiesParseError> {
        let mut result = Self::default();

        match node.attribute((WORD_PROCESSING_XML_NAMESPACE, "val")) {
            Some(val) => result.border_type = val.parse()?,
            None => return Err(BorderPropertiesParseError::ValAttributeMissing)
        }

        if let Some(size) = node.attribute((WORD_PROCESSING_XML_NAMESPACE, "sz")) {
            result.width = EighteenthPoint(size.parse()?);
        }

        if let Some(space) = node.attribute((WORD_PROCESSING_XML_NAMESPACE, "space")) {
            result.spacing = WholePoint(space.parse()?);
        }

        if let Some(color) = node.attribute((WORD_PROCESSING_XML_NAMESPACE, "color")) {
            result.color = color.parse()?;
        }

        Ok(result)
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub enum HexColor {
    #[default]
    Auto,
    Color(crate::gui::Color),
}

#[derive(Debug)]
pub enum HexColorParseError {
    NotSixHexadecimalDigits,
    DigitOutOfRange,
}

impl FromStr for HexColor {
    type Err = HexColorParseError;

    fn from_str(s: &str) -> Result<Self, HexColorParseError> {
        if s == "auto" {
            return Ok(Self::Auto)
        }

        if s.len() != 6 {
            return Err(HexColorParseError::NotSixHexadecimalDigits);
        }

        let mut result = [0u8; 6];
        for i in 0..6 {
            result[i] = match s.as_bytes()[i] {
                b'0'..=b'9' => s.as_bytes()[i] - b'0',
                b'a'..=b'f' => s.as_bytes()[i] - b'a' + 10,
                b'A'..=b'F' => s.as_bytes()[i] - b'A' + 10,
                _ => return Err(HexColorParseError::DigitOutOfRange)
            };
        }

        Ok(Self::Color(crate::gui::Color::from_rgb(
            result[0] * 16 + result[1],
            result[2] * 16 + result[3],
            result[4] * 16 + result[5]
        )))
    }
}

struct Style {
    text_settings: TextSettings,
    table_properties: TableProperties,
}

fn is_correct_namespace(element: &xml::Node) -> bool {
    if element.tag_name().namespace().is_none() {
        return false;
    }

    element.tag_name().namespace().unwrap() == WORD_PROCESSING_XML_NAMESPACE
}

impl Style {

    pub fn from_document_by_style_id(manager: &mut StyleManager, numbering_manager: &crate::wp::numbering::NumberingManager,
                                     theme_settings: &ThemeSettings, document: &xml::Document, name: &str) -> Result<Self, Error> {
        assert!(is_correct_namespace(&document.root_element()));

        for element in document.root_element().children() {
            if !is_correct_namespace(&element) || element.tag_name().name() != "style" {
                continue;
            }

            if let Some(id) = element.attribute((WORD_PROCESSING_XML_NAMESPACE, "styleId")) {
                if id == name {
                    return Self::from_xml(manager, theme_settings, numbering_manager, &element)
                }
            }
        }

        Err(Error::StyleNotFound)
    }

    pub fn from_xml(manager: &mut StyleManager, theme_settings: &ThemeSettings,
            numbering_manager: &crate::wp::numbering::NumberingManager, element: &xml::Node) -> Result<Self, Error> {
        assert!(element.tag_name().namespace().is_some());
        assert_eq!(element.tag_name().namespace().unwrap(), WORD_PROCESSING_XML_NAMESPACE);

        let mut style = Style{
            text_settings: TextSettings::new(),
            table_properties: Default::default(),
        };

        for child in element.children() {
            #[cfg(feature = "debug-styles")]
            println!("Style>> {}", child.tag_name().name());

            if child.tag_name().namespace().is_none() || child.tag_name().namespace().unwrap() != WORD_PROCESSING_XML_NAMESPACE {
                println!("Incorrect namespace: {:?}", child.tag_name().namespace());
                continue;
            }

            match child.tag_name().name() {
                "basedOn" => {
                    let val = child.attribute((WORD_PROCESSING_XML_NAMESPACE, "val"))
                            .expect("No w:val attribute on w:basedOn element!");

                    assert_ne!(element.attribute((WORD_PROCESSING_XML_NAMESPACE, "styleId")).unwrap(), val,
                            "The w:basedOn is used recursively on the same <w:style>! This is an error!");

                    if let Ok(based_on_style) = manager.find_style_using_document(val, element.document(), numbering_manager, theme_settings) {
                        style.inherit_from(based_on_style);
                    }
                }
                "rPr" => {
                    let mut settings = style.text_settings;
                    settings.apply_run_properties_element(manager, theme_settings, &child);
                    style.text_settings = settings;
                }
                "pPr" => {
                    crate::word_processing::process_paragraph_properties_element(numbering_manager, manager,
                        &mut style.text_settings, &child);
                }
                "tblPr" => {
                    style.table_properties = TableProperties::from_xml(&child).unwrap();
                }
                _ => {
                    #[cfg(feature = "debug-styles")]
                    println!("  Unknown");
                }
            }
        }

        Ok(style)
    }

    fn inherit_from(&mut self, style: &Style) {
        self.text_settings = style.text_settings.clone();
    }

}

pub struct StyleManager {
    styles: HashMap<String, Style>,
    default_text_settings: TextSettings,
}

fn process_xml_doc_defaults(element: &xml::Node, manager: &mut StyleManager, theme_settings: &ThemeSettings) {
    for child in element.children() {
        #[cfg(feature = "debug-styles")]
        println!("Style⟫ │  ├─ {}", child.tag_name().name());

        match child.tag_name().name() {
            "rPrDefault" => {
                process_xml_rpr_default(&child, theme_settings, manager);
            }
            "pPrDefault" => {
                process_xml_ppr_default(&child, manager);
            }
            _ => ()
        }
    }
}

fn process_xml_rpr_default(element: &xml::Node, theme_settings: &ThemeSettings, manager: &mut StyleManager) {
    for child in element.children() {
        #[cfg(feature = "debug-styles")]
        println!("Style⟫ │  │  ├─ {}", child.tag_name().name());

        match child.tag_name().name() {
            "rPr" => {
                let mut settings = manager.default_text_settings.clone();

                settings.apply_run_properties_element(manager, theme_settings, &child);

                manager.default_text_settings = settings;
            }
            _ => ()
        }
    }
}

fn process_xml_ppr_default(element: &xml::Node, manager: &mut StyleManager) {
    for child in element.children() {
        #[cfg(feature = "debug-styles")]
        println!("Style⟫ │  │  ├─ {}", child.tag_name().name());

        match child.tag_name().name() {
            "pPr" => {

                // TODO implement this better
                for property in child.children() {
                    match property.tag_name().name() {
                        "spacing" => {
                            if let Some(val) = property.attribute((WORD_PROCESSING_XML_NAMESPACE, "after")) {
                                manager.default_text_settings.spacing_below_paragraph = Some(TwelfteenthPoint(val.parse().unwrap()));
                            }
                        }
                        _ => ()
                    }
                }
            }
            _ => ()
        }
    }
}

impl StyleManager {
    pub fn from_document(document: &xml::Document, numbering_manager: &crate::wp::numbering::NumberingManager,
            theme_settings: &ThemeSettings) -> Result<Self, Error> {
        let mut manager = StyleManager{
            styles: HashMap::new(),
            default_text_settings: TextSettings::new()
        };

        assert_eq!(document.root_element().tag_name().name(), "styles");
        assert!(is_correct_namespace(&document.root_element()));

        #[cfg(feature = "debug-styles")]
        println!("Style⟫ {}", document.root_element().tag_name().name());

        for element in document.root_element().children() {
            #[cfg(feature = "debug-styles")]
            println!("Style⟫ ├─ {}", element.tag_name().name());

            if !is_correct_namespace(&element) {
                continue;
            }

            match element.tag_name().name() {
                "docDefaults" => process_xml_doc_defaults(&element, &mut manager, theme_settings),
                "style" =>
                    match element.attribute((WORD_PROCESSING_XML_NAMESPACE, "styleId")) {
                        Some(id) => {
                            #[cfg(feature = "debug-styles")]
                            println!("Style> {}", id);
                            let style = Style::from_xml(&mut manager, theme_settings, numbering_manager, &element,)?;
                            manager.styles.insert(String::from(id), style);
                        }
                        None => {
                            println!("[Styles] Warning: <w:style> doesn't have a w:styleId attribute!");
                        }
                    }
                _ => ()
            }
        }

        Ok(manager)
    }

    fn find_style_using_document(&mut self, name: &str, document: &xml::Document, numbering_manager: &crate::wp::numbering::NumberingManager,
            theme_settings: &ThemeSettings) -> Result<&Style, Error> {
        if !self.styles.contains_key(name) {
            let style = Style::from_document_by_style_id(self, numbering_manager, theme_settings, document, name)?;

            self.styles.insert(String::from(name), style);
        }

        Ok(self.find_style(name).unwrap())
    }

    fn find_style(&self, name: &str) -> Option<&Style> {
        self.styles.get(name)
    }

    pub fn apply_paragraph_style(&self, style_id: &str, paragraph_text_settings: &mut TextSettings) {
        if let Some(style) = self.styles.get(style_id) {
            paragraph_text_settings.inherit_from(&style.text_settings);
        } else {
            panic!("Style not found: {}", style_id);
        }
    }

    pub fn apply_character_style(&self, style_id: &str, text_settings: &mut TextSettings) {
        if let Some(style) = self.styles.get(style_id) {
            text_settings.inherit_from(&style.text_settings);
        }
    }

    pub fn default_text_settings(&self) -> TextSettings {
        self.default_text_settings.clone()
    }
}
