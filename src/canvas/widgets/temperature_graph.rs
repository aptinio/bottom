use std::borrow::Cow;

use ratatui::{
    Frame,
    layout::{Constraint, Rect},
};

use crate::{
    app::{App, AppConfigFields},
    canvas::{
        Painter,
        components::time_series::{AxisBound, ChartScaling, GraphData, LegendConstraints},
        drawing_utils::should_hide_x_label,
    },
    components::time_series::GraphDrawCtx,
};

impl Painter {
    pub fn draw_temperature_graph(
        &self, f: &mut Frame<'_>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        if let Some(widget_state) = app_state
            .states
            .temp_graph_state
            .get_mut_widget_state(widget_id)
        {
            let shared_data = app_state.data_store.get_data();
            let points = &(shared_data.time_series_data.temperature);
            let times = &(shared_data.time_series_data.time);

            let border_style = self.get_border_style(widget_id, app_state.current_widget.widget_id);
            let hide_x_labels = should_hide_x_label(
                app_state.app_config_fields.hide_time,
                app_state.app_config_fields.autohide_time,
                widget_state.graph.state_mut().autohide_timer_mut(),
                draw_loc,
            );

            let y_max = widget_state.graph.y_max(points.values(), times);
            let (y_bounds, y_labels) = adjust_temp_data_point(
                y_max,
                widget_state.min_temp,
                widget_state.max_temp,
                &app_state.app_config_fields,
            );

            // Hide the legend if the width is 90% of the total widget width
            // or the height is greater than 50% of the total widget height.
            let legend_constraints = LegendConstraints {
                width: Constraint::Ratio(9, 10),
                height: Constraint::Ratio(1, 2),
            };

            let unit = app_state.app_config_fields.temperature_type.unit();
            let graph_data: Vec<GraphData<'_, f32>> = points
                .iter()
                .enumerate()
                .map(|(itx, (source, values))| {
                    // TODO: Maybe align the value later.
                    let name = match values.last() {
                        Some(latest) => format!("{source}: {latest:.0}{unit}").into(),
                        None => source.as_str().into(),
                    };
                    GraphData::default()
                        .name(name)
                        .style(
                            self.styles.temp_graph_colour_styles
                                [itx % self.styles.temp_graph_colour_styles.len()],
                        )
                        .time(times)
                        .values(values)
                })
                .collect();

            let marker = self.get_marker(app_state.app_config_fields.use_dot);

            let y_labels: Vec<Cow<'_, str>> = y_labels.into_iter().map(Into::into).collect();

            widget_state.graph.draw(
                f,
                draw_loc,
                GraphDrawCtx {
                    title: " Temperature ".into(),
                    border_style,
                    title_style: self.styles.widget_title_style,
                    graph_style: self.styles.graph_style,
                    general_widget_style: self.styles.general_widget_style,
                    border_type: self.styles.border_type,
                    marker,
                    hide_x_labels,
                    is_selected: app_state.current_widget.widget_id == widget_id,
                    is_expanded: app_state.is_expanded,
                    legend_position: app_state.app_config_fields.temperature_legend_position,
                    legend_constraints: Some(legend_constraints),
                },
                y_bounds,
                &y_labels,
                ChartScaling::Linear,
                graph_data,
            );
        }

        // Update draw loc in widget map.
        if app_state.should_get_widget_bounds()
            && let Some(temperature_graph_widget) = app_state.widget_map.get_mut(&widget_id)
        {
            temperature_graph_widget.top_left_corner = Some((draw_loc.x, draw_loc.y));
            temperature_graph_widget.bottom_right_corner =
                Some((draw_loc.x + draw_loc.width, draw_loc.y + draw_loc.height));
        }
    }
}

/// Returns the required bounds and labels.
fn adjust_temp_data_point(
    max_entry: f64, min_temp: Option<f64>, max_temp: Option<f64>, config: &AppConfigFields,
) -> (AxisBound, [String; 3]) {
    let default_upper: f64 = config
        .temperature_type
        .convert_temp_unit_float(100.0)
        .into();
    let unit = config.temperature_type.unit();
    let min_entry = min_temp.unwrap_or(0.0);

    let max_entry = max_temp.unwrap_or_else(|| {
        let adaptive_max = max_entry.max(default_upper);
        adaptive_max.max(min_entry + 1.0)
    });

    let halfway = min_entry + (max_entry - min_entry) / 2.0;
    let labels = [
        format!("{:.0}{unit}", min_entry.floor()),
        format!("{:.0}{unit}", halfway.ceil()),
        format!("{:.0}{unit}", max_entry.ceil()),
    ];

    let bounds = if min_temp.is_some() {
        AxisBound::Range(min_entry, max_entry)
    } else {
        AxisBound::Max(max_entry)
    };

    (bounds, labels)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::data::TemperatureType;

    fn config(temperature_type: TemperatureType) -> AppConfigFields {
        AppConfigFields {
            temperature_type,
            ..Default::default()
        }
    }

    #[test]
    fn floors_to_default_upper_when_below() {
        let cfg = config(TemperatureType::Celsius);
        let (bounds, labels) = adjust_temp_data_point(40.0, None, None, &cfg);
        assert_eq!(bounds, AxisBound::Max(100.0));
        assert_eq!(labels, ["0°C", "50°C", "100°C"]);
    }

    #[test]
    fn keeps_actual_max_when_above_default() {
        let cfg = config(TemperatureType::Celsius);
        let (bounds, labels) = adjust_temp_data_point(120.0, None, None, &cfg);
        assert_eq!(bounds, AxisBound::Max(120.0));
        assert_eq!(labels, ["0°C", "60°C", "120°C"]);
    }

    #[test]
    fn lower_limit_keeps_adaptive_upper_bound() {
        let cfg = config(TemperatureType::Celsius);
        let (bounds, labels) = adjust_temp_data_point(40.0, Some(30.0), None, &cfg);
        assert_eq!(bounds, AxisBound::Range(30.0, 100.0));
        assert_eq!(labels, ["30°C", "65°C", "100°C"]);
    }

    #[test]
    fn lower_limit_can_be_negative() {
        let cfg = config(TemperatureType::Celsius);
        let (bounds, labels) = adjust_temp_data_point(40.0, Some(-20.0), None, &cfg);
        assert_eq!(bounds, AxisBound::Range(-20.0, 100.0));
        assert_eq!(labels, ["-20°C", "40°C", "100°C"]);
    }

    #[test]
    fn upper_limit_overrides_actual_max() {
        let cfg = config(TemperatureType::Celsius);
        let (bounds, labels) = adjust_temp_data_point(150.0, None, Some(80.0), &cfg);
        assert_eq!(bounds, AxisBound::Max(80.0));
        assert_eq!(labels, ["0°C", "40°C", "80°C"]);
    }

    #[test]
    fn both_limits_define_a_range() {
        let cfg = config(TemperatureType::Celsius);
        let (bounds, labels) = adjust_temp_data_point(150.0, Some(30.0), Some(80.0), &cfg);
        assert_eq!(bounds, AxisBound::Range(30.0, 80.0));
        assert_eq!(labels, ["30°C", "55°C", "80°C"]);
    }

    #[test]
    fn fahrenheit_default_is_212() {
        let cfg = config(TemperatureType::Fahrenheit);
        let (bounds, labels) = adjust_temp_data_point(100.0, None, None, &cfg);
        assert_eq!(bounds, AxisBound::Max(212.0));
        assert_eq!(labels, ["0°F", "106°F", "212°F"]);
    }

    #[test]
    fn kelvin_default_is_373() {
        let cfg = config(TemperatureType::Kelvin);
        let (bounds, labels) = adjust_temp_data_point(100.0, None, None, &cfg);
        let AxisBound::Max(max) = bounds else {
            panic!("expected a maximum bound");
        };
        assert!((max - 373.15).abs() < 1e-3);
        assert_eq!(labels, ["0K", "187K", "374K"]);
    }
}
