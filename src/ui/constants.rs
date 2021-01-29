// Copyright (c) 2020-2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of Kickmess. See README.md and COPYING for details.

pub fn lighten_clr(depth: u32, clr: (f64, f64, f64)) -> (f64, f64, f64) {
    (clr.0 * (1.2_f64).powf(depth as f64),
     clr.1 * (1.2_f64).powf(depth as f64),
     clr.2 * (1.2_f64).powf(depth as f64))
}
pub const UI_BG_CLR               : (f64, f64, f64) = ( 71.0 / 255.0,   63.0 / 255.0,   73.0 / 255.0);
//pub const UI_BG2_CLR              : (f64, f64, f64) = ( 89.0 / 255.0,   79.0 / 255.0,   93.0 / 255.0);
//pub const UI_BG3_CLR              : (f64, f64, f64) = ( 98.0 / 255.0,   87.0 / 255.0,  102.0 / 255.0);
pub const UI_BG3_CLR              : (f64, f64, f64) = (100.0 / 255.0,   88.0 / 255.0,  104.0 / 255.0);
pub const UI_TXT_CLR              : (f64, f64, f64) = (220.0 / 255.0,  220.0 / 255.0,  240.0 / 255.0);
//pub const UI_LBL_BG_CLR           : (f64, f64, f64) = ( 31.0 / 255.0,   27.0 / 255.0,   33.0 / 255.0);
//pub const UI_LBL_BG_CLR           : (f64, f64, f64) = ( 63.0 / 255.0,   18.0 / 255.0,   61.0 / 255.0);
//pub const UI_BORDER_CLR           : (f64, f64, f64) = ( 54.0 / 255.0,   45.0 / 255.0,   56.0 / 255.0);
//pub const UI_BORDER_CLR           : (f64, f64, f64) = ( 47.0 / 255.0,   39.0 / 255.0,   48.0 / 255.0);
pub const UI_BORDER_CLR           : (f64, f64, f64) = ( 43.0 / 255.0,    5.0 / 255.0,   48.0 / 255.0);
pub const UI_LBL_BG_CLR           : (f64, f64, f64) = ( 32.0 / 255.0,   14.0 / 255.0,   31.0 / 255.0);
pub const UI_ACCENT_CLR           : (f64, f64, f64) = (179.0 / 255.0,   20.0 / 255.0,  170.0 / 255.0);
//pub const UI_ACCENT2_CLR          : (f64, f64, f64) = ( 63.0 / 255.0,   18.0 / 255.0,   61.0 / 255.0);
pub const UI_PRIM_CLR             : (f64, f64, f64) = (105.0 / 255.0,  232.0 / 255.0,  237.0 / 255.0);
pub const UI_PRIM2_CLR            : (f64, f64, f64) = (26.0  / 255.0,  174.0 / 255.0,  179.0 / 255.0);
pub const UI_HLIGHT_CLR           : (f64, f64, f64) = (233.0 / 255.0,  248.0 / 255.0,   64.0 / 255.0);
pub const UI_HLIGHT2_CLR          : (f64, f64, f64) = (181.0 / 255.0,  196.0 / 255.0,   18.0 / 255.0);

pub const UI_INACTIVE_CLR         : (f64, f64, f64) = (111.0 / 255.0,   99.0 / 255.0,  116.0 / 255.0);
pub const UI_INACTIVE2_CLR        : (f64, f64, f64) = (167.0 / 255.0,  148.0 / 255.0,  174.0 / 255.0);

pub const UI_VERSION_FONT_SIZE    : f64 = 10.0;

pub const UI_HELP_FONT_SIZE       : f64 = 16.0;
pub const UI_HELP_TXT_CLR         : (f64, f64, f64) = UI_TXT_CLR;

pub const UI_LBL_TXT_CLR          : (f64, f64, f64) = UI_TXT_CLR;

pub const UI_CONT_FONT_SIZE       : f64 = 14.0;
pub const UI_CONT_FONT_CLR        : (f64, f64, f64) = UI_PRIM_CLR;

pub const UI_BG_KNOB_STROKE       : f64 = 8.0;
pub const UI_MG_KNOB_STROKE       : f64 = 3.0;
pub const UI_FG_KNOB_STROKE       : f64 = 5.0;
pub const UI_BG_KNOB_STROKE_CLR   : (f64, f64, f64) = UI_LBL_BG_CLR;
pub const UI_MG_KNOB_STROKE_CLR   : (f64, f64, f64) = UI_ACCENT_CLR;
pub const UI_FG_KNOB_STROKE_CLR   : (f64, f64, f64) = UI_PRIM_CLR;
pub const UI_TXT_KNOB_CLR         : (f64, f64, f64) = UI_PRIM_CLR;
pub const UI_TXT_KNOB_HOVER_CLR   : (f64, f64, f64) = UI_HLIGHT_CLR;
pub const UI_TXT_KNOB_HLIGHT_CLR  : (f64, f64, f64) = UI_HLIGHT_CLR;
pub const UI_TXT_KNOB_HLHOVR_CLR  : (f64, f64, f64) = UI_HLIGHT2_CLR;
pub const UI_GUI_BG_CLR           : (f64, f64, f64) = UI_BG_CLR;
//pub const UI_GUI_BG2_CLR          : (f64, f64, f64) = UI_BG2_CLR;
//pub const UI_GUI_BG3_CLR          : (f64, f64, f64) = UI_BG3_CLR;
pub const UI_GUI_CLEAR_CLR        : (f64, f64, f64) = UI_LBL_BG_CLR;
pub const UI_BORDER_WIDTH         : f64 = 2.0;
pub const UI_KNOB_RADIUS          : f64 = 25.0;
pub const UI_KNOB_SMALL_RADIUS    : f64 = 14.0;
pub const UI_KNOB_FONT_SIZE       : f64 = 11.0;

pub const UI_BTN_BORDER_CLR       : (f64, f64, f64) = UI_BG_KNOB_STROKE_CLR;
pub const UI_BTN_BORDER2_CLR      : (f64, f64, f64) = UI_ACCENT_CLR;
pub const UI_BTN_BG_CLR           : (f64, f64, f64) = UI_BG_KNOB_STROKE_CLR;
pub const UI_BTN_TXT_CLR          : (f64, f64, f64) = UI_TXT_KNOB_CLR;
pub const UI_BTN_TXT_HOVER_CLR    : (f64, f64, f64) = UI_TXT_KNOB_HOVER_CLR;
pub const UI_BTN_TXT_HLIGHT_CLR   : (f64, f64, f64) = UI_TXT_KNOB_HLIGHT_CLR;
pub const UI_BTN_TXT_HLHOVR_CLR   : (f64, f64, f64) = UI_TXT_KNOB_HLHOVR_CLR;
pub const UI_BTN_WIDTH            : f64 = 3.0 * UI_KNOB_RADIUS;
pub const UI_BTN_BORDER_WIDTH     : f64 = 6.0;
pub const UI_BTN_BORDER2_WIDTH    : f64 = 2.0;
pub const UI_BTN_BEVEL            : f64 = UI_ELEM_TXT_H / 4.0;

pub const UI_GRPH_W               : f64 = 60.0;
pub const UI_GRPH_H               : f64 = 30.0;
pub const UI_GRPH_BORDER          : f64 = 2.0;
pub const UI_GRPH_BORDER_CLR      : (f64, f64, f64) = UI_ACCENT_CLR;
pub const UI_GRPH_FONT_SIZE       : f64 = UI_KNOB_FONT_SIZE;

pub const UI_TAB_WIDTH            : f64 = 90.0;
pub const UI_TAB_FONT_SIZE        : f64 = UI_KNOB_FONT_SIZE;
pub const UI_TAB_BG_CLR           : (f64, f64, f64) = UI_LBL_BG_CLR;
pub const UI_TAB_DIV_WIDTH        : f64 = 2.0;
pub const UI_TAB_DIV_CLR          : (f64, f64, f64) = UI_PRIM_CLR;
pub const UI_TAB_TXT_CLR          : (f64, f64, f64) = UI_PRIM_CLR;
pub const UI_TAB_TXT2_CLR         : (f64, f64, f64) = UI_PRIM2_CLR;
pub const UI_TAB_TXT_HOVER_CLR    : (f64, f64, f64) = UI_BTN_TXT_HOVER_CLR;

pub const UI_BOX_H          : f64 = 200.0;
pub const UI_BOX_BORD       : f64 =   3.0;
pub const UI_MARGIN         : f64 =   4.0;
pub const UI_PADDING        : f64 =   6.0;
pub const UI_ELEM_TXT_H     : f64 =  16.0;
pub const UI_SAFETY_PAD     : f64 =   1.0;

pub const UI_INPUT_BOX_W         : f64 = 200.0;
pub const UI_INPUT_BOX_FONT_SIZE : f64 = 16.0;

pub const UI_DRAG_INFO_W         : f64 = 70.0;
pub const UI_DRAG_INFO_FONT_SIZE : f64 = 10.0;
