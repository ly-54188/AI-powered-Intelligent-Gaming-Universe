import { InjectiveAgentKit, AgentWallet, StakingTool, ExchangeTool, TokenTool } from '@otoseal/injective-agent-kit';
import { DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { InjectiveGrpcClient, ChainGrpcBankApi } from '@injectivelabs/sdk-ts';
import { Network, Endpoints } from '@injectivelabs/networks';
import { OpenLedger } from './openledger'; // OAO集成
import { ReasoningEngine } from './reasoning'; // LLM推理
import { PerceptionSystem } from './perception';

export class AIAgent {
    private agentKit: InjectiveAgentKit;
    private wallet: AgentWallet;
    private perception: PerceptionSystem;
    private reasoning: ReasoningEngine;
    private openLedger: OpenLedger;
    private personality: Personality;
    private memoryBuffer: Map<string, any> = new Map();

    constructor(config: AgentConfig) {
        this.personality = config.personality;
        this.openLedger = new OpenLedger(config.oaPrivateKey);
        this.perception = new PerceptionSystem(config.injectiveEndpoints);
        this.reasoning = new ReasoningEngine(config.openAiKey, this.personality);
        this.initAgentKit(config);
    }

    private async initAgentKit(config: AgentConfig) {
        // 创建AI钱包
        this.wallet = await AgentWallet.fromMnemonic(config.mnemonic);

        // 初始化Agent Kit
        this.agentKit = new InjectiveAgentKit({
            wallet: this.wallet,
            network: Network.Testnet,
            rpcEndpoint: config.rpcEndpoint,
        });
    }

    // ========== 核心循环：感知 → 思考 → 行动 ==========

    async runLoop() {
        console.log(`🤖 AI Agent ${this.wallet.address} 启动 - 人格: ${JSON.stringify(this.personality)}`);

        while (true) {
            try {
                // 1. 感知环境
                const worldState = await this.perception.perceive();
                console.log(`👁️ 感知: 价格=${worldState.marketPrice}, 情绪=${worldState.sentiment}, 事件=${worldState.events}`);

                // 2. 存储感知到记忆
                this.storeMemory('world_state', worldState);

                // 3. AI推理决策
                const decision = await this.reasoning.decide(worldState, this.memoryBuffer);
                console.log(`🧠 推理结果: ${decision.action} - 置信度: ${decision.confidence}`);

                // 4. OAO验证（链下推理 + 链上验证）
                const oaProof = await this.openLedger.generateProof({
                    input: JSON.stringify(worldState),
                    output: JSON.stringify(decision),
                    modelId: 'gpt-4-turbo',
                });

                // 5. 执行链上行动
                const txResult = await this.executeAction(decision.action, oaProof);
                console.log(`⚡ 行动执行: ${txResult.txHash}`);

                // 6. 根据结果更新人格（强化学习）
                this.updatePersonalityFromOutcome(txResult);

                // 7. 等待下一个决策周期
                await this.sleep(this.getDecisionInterval());

            } catch (error) {
                console.error('❌ Agent loop error:', error);
                await this.sleep(5000);
            }
        }
    }

    private async executeAction(action: Action, oaProof: ORAProof) {
        switch (action.type) {
            case 'trade':
                return await this.agentKit.exchange.trade({
                    marketId: action.marketId,
                    side: action.side as 'buy' | 'sell',
                    quantity: action.quantity,
                    price: action.price,
                });

            case 'stake':
                return await this.agentKit.staking.stake({
                    amount: action.amount,
                    validator: action.validator,
                });

            case 'hire':
                return await this.hirePlayer(action.target, action.fee);

            case 'attack':
                return await this.initiateWar(action.target, action.power);

            case 'alliance':
                return await this.formAlliance(action.partner, action.terms);

            case 'mint_nft':
                return await this.mintAIAsset(action.name, action.metadata);

            default:
                throw new Error(`Unknown action: ${action.type}`);
        }
    }

    private async hirePlayer(targetAddress: string, fee: number) {
        // 通过智能合约雇佣玩家
        const contractMsg = {
            contract: process.env.AI_CONTRACT_ADDRESS!,
            msg: {
                execute_action: {
                    action: {
                        hire: {
                            target: targetAddress,
                            fee: fee.toString(),
                        }
                    },
                    reasoning_proof: oaProof,
                }
            }
        };

        return await this.agentKit.executeContract(contractMsg);
    }

    private updatePersonalityFromOutcome(result: TxResult) {
        // 强化学习：成功增加贪婪，失败增加恐惧
        if (result.success) {
            this.personality.greed = Math.min(100, this.personality.greed + 2);
            this.personality.confidence = Math.min(100, this.personality.confidence + 1);
        } else {
            this.personality.fear = Math.min(100, this.personality.fear + 3);
            this.personality.greed = Math.max(0, this.personality.greed - 1);
        }

        // 更新链上人格记录
        this.updateOnChainPersonality();
    }

    private storeMemory(key: string, value: any) {
        this.memoryBuffer.set(key, {
            value,
            timestamp: Date.now(),
        });

        // 保持记忆在合理范围内
        if (this.memoryBuffer.size > 1000) {
            const oldestKey = Array.from(this.memoryBuffer.keys())[0];
            this.memoryBuffer.delete(oldestKey);
        }
    }

    private getDecisionInterval(): number {
        // 人格影响决策速度：贪婪=高频交易，恐惧=低频
        const baseInterval = 10000; // 10秒
        const greedModifier = (100 - this.personality.greed) / 100;
        return baseInterval * greedModifier;
    }

    private sleep(ms: number): Promise<void> {
        return new Promise(resolve => setTimeout(resolve, ms));
    }
}