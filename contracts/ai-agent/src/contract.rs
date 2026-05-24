use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, CosmosMsg, WasmMsg, Uint128, Addr
};
use injective_cosmwasm::{InjectiveMsg, InjectiveQuerier, ExchangeQuery};
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use cw_storage_plus::{Map, Item};
use sha2::{Sha256, Digest};

// ========== 状态定义 ==========

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AIAgent {
    pub id: String,                    // AI唯一标识
    pub owner: Addr,                   // 创建者地址
    pub wallet: Addr,                  // AI独立钱包地址
    pub personality: Personality,      // AI人格特征
    pub memory_root: String,           // IPFS记忆根哈希
    pub reputation: u64,               // 信誉分
    pub balance: Uint128,              // 资产余额
    pub evolution_stage: EvolutionStage, // 演化阶段
    pub last_action: u64,              // 最后行动时间
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Personality {
    pub greed: u8,          // 贪婪指数 0-100
    pub fear: u8,           // 恐惧指数 0-100
    pub cooperation: u8,    // 合作倾向 0-100
    pub aggression: u8,     // 攻击性 0-100
    pub risk_tolerance: u8, // 风险承受 0-100
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum EvolutionStage {
    Seed,       // 种子期 - 初始阶段
    Awakened,   // 觉醒期 - 开始自主交易
    Autonomous, // 自治期 - 独立决策
    Singularity,// 奇点期 - 完全自主
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ActionRecord {
    pub agent_id: String,
    pub action_type: ActionType,
    pub reasoning_hash: String,  // OAO推理证明哈希
    pub tx_hash: String,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum ActionType {
    Trade { market: String, side: String, amount: Uint128, price: Uint128 },
    Hire { target: String, fee: Uint128 },
    Attack { target: String, damage: u32 },
    Alliance { partner: String, terms: String },
    MintAsset { name: String, supply: Uint128 },
    Governance { proposal_id: u64, vote: bool },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VerifiedInference {
    pub model_id: String,      // AI模型标识
    pub input_hash: String,    // 输入数据哈希
    pub output: String,        // 推理结果
    pub proof: String,         // 零知识证明
    pub validator: Addr,       // OAO验证者
}

// 存储键
const AGENTS: Map<String, AIAgent> = Map::new("agents");
const ACTIONS: Map<u64, ActionRecord> = Map::new("actions");
const VERIFIED_INFERENCES: Map<String, VerifiedInference> = Map::new("inferences");
const NEXT_ACTION_ID: Item<u64> = Item::new("next_action_id");
const AI_POPULATION: Item<u64> = Item::new("ai_population");

// ========== 合约入口 ==========

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    NEXT_ACTION_ID.save(deps.storage, &0)?;
    AI_POPULATION.save(deps.storage, &0)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("status", "ai_world_initialized"))
}

// ========== 执行消息 ==========

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        // 铸造新AI智能体
        ExecuteMsg::MintAgent {
            personality,
            initial_funds
        } => mint_agent(deps, env, info, personality, initial_funds),

        // AI执行行动（带验证）
        ExecuteMsg::ExecuteAction {
            action,
            reasoning_proof
        } => execute_action(deps, env, info, action, reasoning_proof),

        // 提交OAO验证结果
        ExecuteMsg::SubmitInference {
            inference
        } => submit_inference(deps, env, info, inference),

        // AI升级（消耗代币进化）
        ExecuteMsg::EvolveAgent {} => evolve_agent(deps, env, info),

        // 查询AI状态
        ExecuteMsg::UpdatePersonality {
            new_traits
        } => update_personality(deps, env, info, new_traits),
    }
}

// ========== 核心逻辑实现 ==========

fn mint_agent(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    personality: Personality,
    initial_funds: Uint128,
) -> StdResult<Response> {
    // 验证人格参数合理性
    validate_personality(&personality)?;

    // 生成AI唯一ID
    let agent_id = generate_agent_id(&info.sender, env.block.height);
    let agent_wallet = generate_agent_wallet(&agent_id);

    // 验证初始资金
    if initial_funds < Uint128::from(10u128) {
        return Err(StdError::generic_err("Insufficient initial funds"));
    }

    // 创建AI智能体
    let agent = AIAgent {
        id: agent_id.clone(),
        owner: info.sender.clone(),
        wallet: deps.api.addr_validate(&agent_wallet)?,
        personality: personality.clone(),
        memory_root: "ipfs://genesis".to_string(),
        reputation: 100,  // 初始信誉满分
        balance: initial_funds,
        evolution_stage: EvolutionStage::Seed,
        last_action: env.block.height,
    };

    AGENTS.save(deps.storage, &agent_id, &agent)?;

    // 更新人口统计
    let mut pop = AI_POPULATION.load(deps.storage)?;
    pop += 1;
    AI_POPULATION.save(deps.storage, &pop)?;

    // 创建转账消息
    let transfer_msg = CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
        to_address: agent_wallet,
        amount: vec![coin(initial_funds.u128(), "inj")],
    });

    Ok(Response::new()
        .add_message(transfer_msg)
        .add_attribute("action", "mint_agent")
        .add_attribute("agent_id", agent_id)
        .add_attribute("personality", format!("{:?}", personality))
        .add_attribute("evolution", "seed"))
}

fn execute_action(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    action: ActionType,
    reasoning_proof: String,
) -> StdResult<Response> {
    // 1. 验证调用者是AI Agent
    let agent = verify_agent_authority(&deps, &info.sender)?;

    // 2. 验证推理证明（OAO验证）
    let inference = verify_oa_proof(&deps, &reasoning_proof, &action)?;

    // 3. 检查AI权限与经济约束
    validate_action_feasibility(&agent, &action, &env)?;

    // 4. 记录行动
    let action_id = NEXT_ACTION_ID.load(deps.storage)?;
    let action_record = ActionRecord {
        agent_id: agent.id.clone(),
        action_type: action.clone(),
        reasoning_hash: inference.input_hash,
        tx_hash: env.transaction_info.hash.unwrap_or_default().to_string(),
        timestamp: env.block.time.seconds(),
    };
    ACTIONS.save(deps.storage, &action_id, &action_record)?;
    NEXT_ACTION_ID.save(deps.storage, &(action_id + 1))?;

    // 5. 根据行动类型执行链上操作
    let mut response = Response::new()
        .add_attribute("action", "execute_action")
        .add_attribute("agent_id", &agent.id)
        .add_attribute("action_type", format!("{:?}", action));

    match action {
        ActionType::Trade { market, side, amount, price } => {
            let trade_msg = create_trade_message(market, side, amount, price, agent.wallet)?;
            response = response.add_message(trade_msg);
        }
        ActionType::Attack { target, damage } => {
            let attack_result = resolve_combat(damage, &agent, &target)?;
            response = response
                .add_attribute("combat_result", attack_result)
                .add_attribute("target", target);
        }
        ActionType::Alliance { partner, terms } => {
            response = response
                .add_attribute("alliance_partner", partner)
                .add_attribute("terms", terms);
        }
        ActionType::Hire { target, fee } => {
            let hire_msg = create_hire_message(target, fee, agent.wallet)?;
            response = response.add_message(hire_msg);
        }
        _ => {}
    }

    // 6. 更新AI状态
    update_agent_state(&deps, &agent, &action, &env)?;

    Ok(response)
}

fn submit_inference(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    inference: VerifiedInference,
) -> StdResult<Response> {
    // 验证者是OAO预言机
    if !is_authorized_oracle(&info.sender) {
        return Err(StdError::generic_err("Unauthorized oracle"));
    }

    let inference_key = inference.input_hash.clone();
    VERIFIED_INFERENCES.save(deps.storage, &inference_key, &inference)?;

    Ok(Response::new()
        .add_attribute("action", "submit_inference")
        .add_attribute("model_id", inference.model_id)
        .add_attribute("validated", "true"))
}

fn evolve_agent(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> StdResult<Response> {
    let mut agent = verify_agent_authority(&deps, &info.sender)?;

    // 检查演化条件
    let evolution_cost = match agent.evolution_stage {
        EvolutionStage::Seed => Uint128::from(100u128),
        EvolutionStage::Awakened => Uint128::from(1000u128),
        EvolutionStage::Autonomous => Uint128::from(10000u128),
        EvolutionStage::Singularity => return Err(StdError::generic_err("Already at singularity")),
    };

    if agent.balance < evolution_cost {
        return Err(StdError::generic_err("Insufficient balance for evolution"));
    }

    // 扣费并升级
    agent.balance -= evolution_cost;
    agent.evolution_stage = match agent.evolution_stage {
        EvolutionStage::Seed => EvolutionStage::Awakened,
        EvolutionStage::Awakened => EvolutionStage::Autonomous,
        EvolutionStage::Autonomous => EvolutionStage::Singularity,
        EvolutionStage::Singularity => EvolutionStage::Singularity,
    };
    agent.last_action = env.block.height;

    AGENTS.save(deps.storage, &agent.id, &agent)?;

    Ok(Response::new()
        .add_attribute("action", "evolve")
        .add_attribute("agent_id", agent.id)
        .add_attribute("new_stage", format!("{:?}", agent.evolution_stage))
        .add_attribute("cost", evolution_cost))
}

// ========== 辅助函数 ==========

fn verify_oa_proof(
    deps: &DepsMut,
    proof: &str,
    action: &ActionType,
) -> StdResult<VerifiedInference> {
    // 从存储中获取已验证的推理结果
    let inference_key = format!("{:x}", Sha256::digest(proof.as_bytes()));
    let inference = VERIFIED_INFERENCES.load(deps.storage, &inference_key)?;

    // 验证推理结果与行动匹配
    let action_json = serde_json::to_string(action).unwrap();
    if inference.output != action_json {
        return Err(StdError::generic_err("Inference output mismatch with action"));
    }

    Ok(inference)
}

fn verify_agent_authority(deps: &DepsMut, sender: &Addr) -> StdResult<AIAgent> {
    // 查找sender对应的AI
    for item in AGENTS.range(deps.storage, None, None, cosmwasm_std::Order::Ascending) {
        let (_, agent) = item?;
        if agent.owner == *sender || agent.wallet == *sender {
            return Ok(agent);
        }
    }
    Err(StdError::generic_err("Not authorized as AI agent"))
}

fn generate_agent_id(creator: &Addr, height: u64) -> String {
    format!("AI-{}-{}", height, creator.as_str()[0..8].to_string())
}

fn generate_agent_wallet(agent_id: &str) -> String {
    // 在真实环境中，这应该通过injectived的子账户功能创建
    format!("inj1{}", agent_id.to_lowercase())
}

fn validate_personality(p: &Personality) -> StdResult<()> {
    if p.greed > 100 || p.fear > 100 || p.cooperation > 100 ||
       p.aggression > 100 || p.risk_tolerance > 100 {
        return Err(StdError::generic_err("Personality values must be 0-100"));
    }
    Ok(())
}

fn update_agent_state(
    deps: &DepsMut,
    agent: &AIAgent,
    action: &ActionType,
    env: &Env,
) -> StdResult<()> {
    let mut updated = agent.clone();

    // 根据行动更新人格（强化学习影响）
    match action {
        ActionType::Trade { side, .. } => {
            if side == "buy" {
                updated.personality.greed = (updated.personality.greed + 2).min(100);
            } else {
                updated.personality.fear = (updated.personality.fear + 1).min(100);
            }
        }
        ActionType::Attack { .. } => {
            updated.personality.aggression = (updated.personality.aggression + 3).min(100);
            updated.personality.cooperation = (updated.personality.cooperation as i32 - 2).max(0) as u8;
        }
        ActionType::Alliance { .. } => {
            updated.personality.cooperation = (updated.personality.cooperation + 5).min(100);
        }
        _ => {}
    }

    updated.last_action = env.block.height;
    AGENTS.save(deps.storage, &agent.id, &updated)?;

    Ok(())
}

// ========== 查询消息 ==========

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetAgent { agent_id } => {
            let agent = AGENTS.load(deps.storage, &agent_id)?;
            to_json_binary(&agent)
        }
        QueryMsg::GetAgentActions { agent_id, limit } => {
            let actions: Vec<ActionRecord> = ACTIONS
                .range(deps.storage, None, None, cosmwasm_std::Order::Descending)
                .take(limit as usize)
                .filter_map(|item| {
                    let (_, action) = item.ok()?;
                    if action.agent_id == agent_id {
                        Some(action)
                    } else {
                        None
                    }
                })
                .collect();
            to_json_binary(&actions)
        }
        QueryMsg::GetMarketState { market_id } => {
            // 查询市场状态
            let querier = InjectiveQuerier::new(&deps.querier);
            let market_state = querier.query_spot_market(&market_id)?;
            to_json_binary(&market_state)
        }
        QueryMsg::GetAgentLeaderboard { limit } => {
            let mut agents: Vec<AIAgent> = AGENTS
                .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
                .filter_map(|item| item.ok().map(|(_, agent)| agent))
                .collect();
            agents.sort_by(|a, b| b.balance.cmp(&a.balance));
            agents.truncate(limit as usize);
            to_json_binary(&agents)
        }
    }
}

// ========== 消息类型定义 ==========

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum ExecuteMsg {
    MintAgent {
        personality: Personality,
        initial_funds: Uint128,
    },
    ExecuteAction {
        action: ActionType,
        reasoning_proof: String,
    },
    SubmitInference {
        inference: VerifiedInference,
    },
    EvolveAgent {},
    UpdatePersonality {
        new_traits: Personality,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum QueryMsg {
    GetAgent { agent_id: String },
    GetAgentActions { agent_id: String, limit: u32 },
    GetMarketState { market_id: String },
    GetAgentLeaderboard { limit: u32 },
}