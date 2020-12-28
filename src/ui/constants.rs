// Copyright (c) 2020-2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of Kickmess. See README.md and COPYING for details.

pub const UI_BG_CLR               : (f64, f64, f64) = (46.0  / 255.0,   41.0 / 255.0,   47.0 / 255.0);
pub const UI_BG2_CLR              : (f64, f64, f64) = (57.0 / 255.0,   50.0 / 255.0,  59.0 / 255.0);
pub const UI_LBL_BG_CLR           : (f64, f64, f64) = (21.0  / 255.0,   18.0 / 255.0,   22.0 / 255.0);
pub const UI_ACCENT_CLR           : (f64, f64, f64) = (143.0 / 255.0,   16.0 / 255.0,  136.0 / 255.0);
pub const UI_PRIM_CLR             : (f64, f64, f64) = (105.0 / 255.0,  232.0 / 255.0,  237.0 / 255.0);
pub const UI_PRIM2_CLR            : (f64, f64, f64) = (26.0  / 255.0,  174.0 / 255.0,  179.0 / 255.0);
pub const UI_HLIGHT_CLR           : (f64, f64, f64) = (233.0 / 255.0,  248.0 / 255.0,   64.0 / 255.0);
pub const UI_HLIGHT2_CLR          : (f64, f64, f64) = (181.0 / 255.0,  196.0 / 255.0,   18.0 / 255.0);

pub const UI_HELP_FONT_SIZE       : f64 = 16.0;
pub const UI_HELP_TXT_CLR         : (f64, f64, f64) = (200.0 / 255.0,  200.0 / 255.0,  200.0 / 255.0);

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
pub const UI_GUI_BG2_CLR          : (f64, f64, f64) = UI_BG2_CLR;
pub const UI_GUI_CLEAR_CLR        : (f64, f64, f64) = UI_LBL_BG_CLR;
pub const UI_BORDER_CLR           : (f64, f64, f64) = UI_ACCENT_CLR;
pub const UI_BORDER_WIDTH         : f64 = 2.0;
pub const UI_KNOB_RADIUS          : f64 = 30.0;
pub const UI_KNOB_SMALL_RADIUS    : f64 = 20.0;
pub const UI_KNOB_FONT_SIZE       : f64 = 12.0;

pub const UI_BTN_BORDER_CLR       : (f64, f64, f64) = UI_BG_KNOB_STROKE_CLR;
pub const UI_BTN_BORDER2_CLR      : (f64, f64, f64) = UI_BORDER_CLR;
pub const UI_BTN_BG_CLR           : (f64, f64, f64) = UI_BG_KNOB_STROKE_CLR;
pub const UI_BTN_TXT_CLR          : (f64, f64, f64) = UI_TXT_KNOB_CLR;
pub const UI_BTN_TXT_HOVER_CLR    : (f64, f64, f64) = UI_TXT_KNOB_HOVER_CLR;
pub const UI_BTN_TXT_HLIGHT_CLR   : (f64, f64, f64) = UI_TXT_KNOB_HLIGHT_CLR;
pub const UI_BTN_TXT_HLHOVR_CLR   : (f64, f64, f64) = UI_TXT_KNOB_HLHOVR_CLR;
pub const UI_BTN_WIDTH            : f64 = 3.0 * UI_KNOB_RADIUS;
pub const UI_BTN_BORDER_WIDTH     : f64 = 6.0;
pub const UI_BTN_BORDER2_WIDTH    : f64 = 2.0;
pub const UI_BTN_BEVEL            : f64 = UI_ELEM_TXT_H / 4.0;

pub const UI_GRPH_W               : f64 = 90.0;
pub const UI_GRPH_H               : f64 = 40.0;
pub const UI_GRPH_BORDER          : f64 = 2.0;
pub const UI_GRPH_BORDER_CLR      : (f64, f64, f64) = UI_BORDER_CLR;
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
pub const UI_ELEM_TXT_H     : f64 =  20.0;
pub const UI_SAFETY_PAD     : f64 =   1.0;
