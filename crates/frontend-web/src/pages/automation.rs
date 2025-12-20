
use leptos::*;
use crate::components::node_graph::NodeGraph;

#[component]
pub fn AutomationPage() -> impl IntoView {
    view! {
        <div class="h-screen w-full flex flex-col">
            <header class="bg-white border-b border-gray-200 p-4 flex justify-between items-center">
                <h1 class="text-xl font-bold text-gray-800">Automation Editor</h1>
                <div class="space-x-2">
                    <button class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700">"Save Workflow"</button>
                </div>
            </header>
            <main class="flex-1 overflow-hidden">
                <NodeGraph />
            </main>
        </div>
    }
}
