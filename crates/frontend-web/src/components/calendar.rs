//! Calendar View Component - Metadata-driven calendar view
//! Displays records by date field on a weekly calendar

use leptos::*;
use serde::{Deserialize, Serialize};
use chrono::{Datelike, Duration, NaiveDate};
use crate::api::fetch_entity_list;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalendarConfig {
    pub date_field: String,
    pub end_date_field: Option<String>,
    pub title_field: String,
    pub color_field: Option<String>,
    pub color_map: Option<std::collections::HashMap<String, String>>,
}

#[derive(Clone, Debug)]
pub struct CalendarEvent {
    pub id: String,
    pub title: String,
    pub date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub color: String,
    pub record: serde_json::Value,
}

#[component]
pub fn CalendarView(
    entity_type: String,
    config: CalendarConfig,
) -> impl IntoView {
    let _entity_type_stored = store_value(entity_type.clone());
    let config_stored = store_value(config);
    
    // State
    let (events, set_events) = create_signal::<Vec<CalendarEvent>>(Vec::new());
    let (loading, set_loading) = create_signal(true);
    let (error, set_error) = create_signal::<Option<String>>(None);
    let (current_week_start, set_current_week_start) = create_signal(get_week_start(chrono::Local::now().date_naive()));
    
    // Fetch records
    let entity_for_effect = entity_type.clone();
    create_effect(move |_| {
        let _ = current_week_start.get(); // React to week changes
        let et = entity_for_effect.clone();
        let cfg = config_stored.get_value();
        
        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);
            
            match fetch_entity_list(&et).await {
                Ok(response) => {
                    let records = response.data;
                    let calendar_events: Vec<CalendarEvent> = records.into_iter()
                        .filter_map(|record| {
                            // Parse date from record
                            let date_str = record.get(&cfg.date_field)
                                .and_then(|v| v.as_str())?;
                            
                            // Try to parse date (supports YYYY-MM-DD or ISO datetime)
                            let date = parse_date(date_str)?;
                            
                            let end_date = cfg.end_date_field.as_ref()
                                .and_then(|f| record.get(f))
                                .and_then(|v| v.as_str())
                                .and_then(|s| parse_date(s));
                            
                            let title = record.get(&cfg.title_field)
                                .and_then(|v| v.as_str())
                                .unwrap_or("Untitled")
                                .to_string();
                            
                            let color = cfg.color_field.as_ref()
                                .and_then(|f| record.get(f))
                                .and_then(|v| v.as_str())
                                .and_then(|status| cfg.color_map.as_ref()?.get(status))
                                .cloned()
                                .unwrap_or_else(|| "#3b82f6".to_string());
                            
                            let id = record.get("id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            
                            Some(CalendarEvent {
                                id,
                                title,
                                date,
                                end_date,
                                color,
                                record,
                            })
                        })
                        .collect();
                    
                    set_events.set(calendar_events);
                    set_loading.set(false);
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    });
    
    // Get days for current week
    let week_days = move || {
        let start = current_week_start.get();
        (0..7).map(|i| start + Duration::days(i)).collect::<Vec<_>>()
    };
    
    // Get events for a particular day
    let events_for_day = move |day: NaiveDate| {
        events.get()
            .into_iter()
            .filter(|e| {
                if let Some(end) = e.end_date {
                    e.date <= day && end >= day
                } else {
                    e.date == day
                }
            })
            .collect::<Vec<_>>()
    };
    
    view! {
        <div class="calendar-container">
            <div class="calendar-header">
                <button 
                    class="nav-btn"
                    on:click=move |_| {
                        set_current_week_start.update(|d| *d = *d - Duration::days(7));
                    }
                >
                    "← Prev"
                </button>
                <span class="week-label">
                    {move || {
                        let start = current_week_start.get();
                        let end = start + Duration::days(6);
                        format!("{} - {}", start.format("%b %d"), end.format("%b %d, %Y"))
                    }}
                </span>
                <button 
                    class="nav-btn today-btn"
                    on:click=move |_| {
                        set_current_week_start.set(get_week_start(chrono::Local::now().date_naive()));
                    }
                >
                    "Today"
                </button>
                <button 
                    class="nav-btn"
                    on:click=move |_| {
                        set_current_week_start.update(|d| *d = *d + Duration::days(7));
                    }
                >
                    "Next →"
                </button>
            </div>
            
            {move || {
                if loading.get() {
                    view! { <div class="calendar-loading">"Loading..."</div> }.into_view()
                } else if let Some(err) = error.get() {
                    view! { <div class="calendar-error">{err}</div> }.into_view()
                } else {
                    view! {
                        <div class="calendar-grid">
                            // Day headers
                            <For
                                each=move || week_days()
                                key=|d| d.to_string()
                                children=move |day| {
                                    let is_today = day == chrono::Local::now().date_naive();
                                    view! {
                                        <div class=format!("calendar-day-header {}", if is_today { "today" } else { "" })>
                                            <span class="day-name">{day.format("%a").to_string()}</span>
                                            <span class="day-num">{day.day()}</span>
                                        </div>
                                    }
                                }
                            />
                            
                            // Day cells with events
                            <For
                                each=move || week_days()
                                key=|d| d.to_string()
                                children=move |day| {
                                    let day_events = events_for_day(day);
                                    let is_today = day == chrono::Local::now().date_naive();
                                    view! {
                                        <div class=format!("calendar-day-cell {}", if is_today { "today" } else { "" })>
                                            {day_events.into_iter().map(|event| {
                                                let bg_color = event.color.clone();
                                                view! {
                                                    <div 
                                                        class="calendar-event"
                                                        style=format!("background-color: {}", bg_color)
                                                    >
                                                        <span class="event-title">{event.title}</span>
                                                    </div>
                                                }
                                            }).collect_view()}
                                        </div>
                                    }
                                }
                            />
                        </div>
                    }.into_view()
                }
            }}
        </div>
    }
}

fn get_week_start(date: NaiveDate) -> NaiveDate {
    let days_from_sunday = date.weekday().num_days_from_sunday() as i64;
    date - Duration::days(days_from_sunday)
}

fn parse_date(s: &str) -> Option<NaiveDate> {
    // Try YYYY-MM-DD
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Some(d);
    }
    // Try ISO datetime
    if let Some(date_part) = s.split('T').next() {
        if let Ok(d) = NaiveDate::parse_from_str(date_part, "%Y-%m-%d") {
            return Some(d);
        }
    }
    None
}
