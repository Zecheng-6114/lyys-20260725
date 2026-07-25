pub struct NpcDef {
    pub id: String,
    pub name: String,
    pub dialogue_id: String,
}

pub struct DialogueDef {
    pub id: String,
    pub npc_name: String,
    pub nodes: Vec<DialogueNode>,
}

pub struct DialogueNode {
    pub id: String,
    pub text: String,
    pub choices: Vec<DialogueChoice>,
}

#[derive(Clone)]
pub struct DialogueChoice {
    pub text: String,
    pub next: Option<String>,
    pub effects: Vec<DialogueEffect>,
}

#[derive(Clone)]
pub enum DialogueEffect {
    GiveGold(u32),
    GiveItem { def_id: String, quantity: u32 },
    TakeGold(u32),
    TakeItem { def_id: String, quantity: u32 },
}

pub struct ActiveDialogue {
    pub dialogue_id: String,
    pub current_node_id: String,
}
