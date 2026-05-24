import React, { useEffect, useState } from 'react';
import { InjectiveWalletConnector, WalletStrategy } from '@injectivelabs/wallet-ts';
import { AIWorldDashboard } from './components/AIWorldDashboard';
import { AgentMarketplace } from './components/AgentMarketplace';
import { AgentEvolution } from './components/AgentEvolution';
import { WorldEconomy } from './components/WorldEconomy';

function App() {
    const [wallet, setWallet] = useState(null);
    const [agents, setAgents] = useState([]);
    const [marketData, setMarketData] = useState(null);

    useEffect(() => {
        // 初始化钱包连接
        const initWallet = async () => {
            const strategy = new WalletStrategy({
                chainId: 'injective-888', // testnet
                wallet: new InjectiveWalletConnector(),
            });
            await strategy.connect();
            setWallet(strategy);
        };
        initWallet();

        // 订阅AI世界状态
        subscribeToWorldState();
    }, []);

    const subscribeToWorldState = () => {
        // WebSocket订阅实时数据
        const ws = new WebSocket('wss://sentry.testnet.injective.network/ws');

        ws.onmessage = (event) => {
            const data = JSON.parse(event.data);
            if (data.type === 'agent_action') {
                updateAgentActivity(data.agent);
            }
            if (data.type === 'market_update') {
                setMarketData(data.market);
            }
        };
    };

    return (
        <div className="min-h-screen bg-gradient-to-b from-gray-900 to-black text-white">
            <header className="border-b border-purple-500/20 p-4">
                <div className="container mx-auto flex justify-between items-center">
                    <h1 className="text-2xl font-bold bg-gradient-to-r from-purple-400 to-pink-400 bg-clip-text text-transparent">
                        AI 智能化游戏大世界
                    </h1>
                    <div className="flex gap-4">
                        <span className="text-sm text-gray-400">
                            🌍 AI人口: {agents.length}
                        </span>
                        <button
                            onClick={() => wallet?.disconnect()}
                            className="px-4 py-2 bg-purple-600 rounded-lg hover:bg-purple-700"
                        >
                            连接钱包
                        </button>
                    </div>
                </div>
            </header>

            <main className="container mx-auto p-6">
                <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
                    {/* AI智能体市场 */}
                    <AgentMarketplace agents={agents} />

                    {/* 世界经济面板 */}
                    <WorldEconomy marketData={marketData} />

                    {/* AI进化追踪器 */}
                    <AgentEvolution agents={agents} />
                </div>

                {/* 主世界视图 */}
                <AIWorldDashboard
                    agents={agents}
                    onMintAgent={handleMintAgent}
                    onInteract={handleAgentInteraction}
                />
            </main>
        </div>
    );
}

export default App;