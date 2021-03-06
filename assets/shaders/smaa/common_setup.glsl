// SMAA Common Parameters and Macros

#define mad(x, y, z) (x * y + z)

// texture access defines
#define areatex_select(sample) sample.rg
#define searchtex_select(sample) sample.r

// Adjustable Variables //
// SMAA Presets: Adjusted to High
#define SMAA_THRES 0.1f
#define SMAA_MAX_SEARCH_STEPS 16
#define SMAA_MAX_SEARCH_STEPS_DIAG 8
#define SMAA_CORNER_ROUNDING 25

#define SMAA_LOCAL_CONTRAST_ADAPTION_FACTOR 2.0f

// Non-adjustable Constants //
const int SMAA_AREATEX_MAX_DISTANCE = 16;
const int SMAA_AREATEX_MAX_DISTANCE_DIAG = 20;
const vec2 SMAA_AREATEX_PIXEL_SIZE = 1.0f / vec2(160.0f, 560.0f);
const float SMAA_AREATEX_SUBTEX_SIZE = 1.0f / 7.0f;
const vec2 SMAA_SEARCHTEX_SIZE = vec2(66.0f, 33.0f);
const vec2 SMAA_SEARCHTEX_PACKED_SIZE = vec2(64.0f, 16.0f);
const float SMAA_CORNER_ROUNDING_NORM = float(SMAA_CORNER_ROUNDING) / 100.0f;

// Specialized Constants //
layout(constant_id = 0) const float viewport_width = 640.0f;
layout(constant_id = 1) const float viewport_height = 480.0f;
layout(constant_id = 2) const float viewport_width_rcp = 1.0f / 640.0f;
layout(constant_id = 3) const float viewport_height_rcp = 1.0f / 480.0f;
#define rt_metrics vec4(viewport_width_rcp, viewport_height_rcp, viewport_width, viewport_height)
