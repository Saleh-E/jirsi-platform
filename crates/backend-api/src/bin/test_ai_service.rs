use backend_api::ai::service::create_ai_service;
use core_node_engine::{ExecutionContext, NodeHandler};
use core_node_engine::nodes::AiGenerateHandler;
use core_models::{NodeDef, NodeType};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize AI Service (Mock)
    // We expect this to default to mock because we are not setting OPENAI_API_KEY
    println!("Initializing AI Service...");
    let ai_service = create_ai_service();
    
    // 2. Setup Execution Context
    println!("Setting up Execution Context...");
    let mut context = ExecutionContext::new();
    context = context.with_ai_service(ai_service);
    
    // Add some trigger data to test variable substitution
    context.values.insert("$trigger".to_string(), json!({
        "description": "A very important lead",
        "source": "referral"
    }));

    // 3. Configure Node
    let node_def = NodeDef {
        id: Uuid::new_v4(),
        graph_id: Uuid::new_v4(),
        node_type: NodeType::AiGenerate,
        label: "AI Node".to_string(),
        x: 0.0,
        y: 0.0,
        config: json!({
            "prompt": "Summarize this: {{trigger.description}}. Source: {{trigger.source}}",
            "system_prompt": "You are a helpful assistant."
        }),
        is_enabled: true,
    };
    
    // 4. Instantiate Handler
    let handler = AiGenerateHandler;
    
    // 5. Execute
    println!("Executing AI Node...");
    println!("Prompt Template: {}", node_def.config["prompt"]);
    
    let inputs = HashMap::new();
    let result = handler.execute(&node_def, inputs, &mut context).await?;
    
    // 6. Output Result
    println!("\n--- Execution Result ---");
    println!("{}", serde_json::to_string_pretty(&result)?);
    println!("------------------------\n");
    
    // Verify specific output (Mock response)
    /* 
       The mock service in openai.rs returns: 
       "Mock response for: " + prompt
    */
    if let Some(text) = result.get("text").and_then(|t| t.as_str()) {
        if text.contains("Mock response for") {
            println!("SUCCESS: Received mock response.");
        } else if text.len() > 0 {
             println!("SUCCESS: Received response (Real API active?)");
        } else {
            println!("FAILURE: Empty response text.");
        }
    } else {
        println!("FAILURE: Output missing 'text' field.");
    }

    Ok(())
}
