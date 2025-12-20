
use leptos::*;
use leptos::svg::*;
// use core_node_engine::nodes::NodeConfig; 

#[component]
pub fn GraphNode(
    x: f64, 
    y: f64, 
    label: String,
    #[prop(default = "#1e293b".to_string())] color: String
) -> impl IntoView {
    view! {
        <g transform=format!("translate({}, {})", x, y)>
            <rect width="150" height="80" rx="4" fill=color stroke="#334155" stroke-width="2" />
            <text x="10" y="25" fill="white" class="text-sm font-bold pointer-events-none select-none">{label}</text>
            // Input port
            <circle cx="0" cy="40" r="5" fill="#3b82f6" class="cursor-pointer hover:fill-blue-400" />
            // Output port
            <circle cx="150" cy="40" r="5" fill="#22c55e" class="cursor-pointer hover:fill-green-400"/>
        </g>
    }
}

#[component]
pub fn NodeGraph() -> impl IntoView {
    let (nodes, set_nodes) = create_signal(vec![
        (100.0, 100.0, "Trigger: Contact Created".to_string(), "#4f46e5".to_string()),
        (350.0, 150.0, "Action: Send Email".to_string(), "#0f172a".to_string()),
        (350.0, 300.0, "Action: Create Task".to_string(), "#0f172a".to_string()),
    ]);

    view! {
        <div class="h-full w-full bg-slate-900 overflow-hidden relative border border-slate-700 rounded-lg">
            <svg class="w-full h-full cursor-grab">
                <defs>
                    <pattern id="grid" width="40" height="40" patternUnits="userSpaceOnUse">
                        <path d="M 40 0 L 0 0 0 40" fill="none" stroke="rgba(255,255,255,0.05)" stroke-width="1"/>
                    </pattern>
                </defs>
                <rect width="100%" height="100%" fill="url(#grid)" />
                
                <For
                    each=move || nodes.get()
                    key=|n| format!("{}{}", n.0, n.1)
                    children=move |(x, y, label, color)| {
                        view! {
                            <GraphNode x=x y=y label=label color=color />
                        }
                    }
                />
            </svg>
            <div class="absolute top-4 right-4 bg-slate-800 p-2 rounded shadow text-xs text-slate-400">
                Offline Mode: Ready
            </div>
        </div>
    }
}
