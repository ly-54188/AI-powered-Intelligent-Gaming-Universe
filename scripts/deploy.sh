#!/bin/bash

# 设置环境变量
export INJECTIVE_NETWORK="testnet"
export CONTRACT_WASM="./artifacts/ai_agent.wasm"

echo "🚀 部署 AI 游戏大世界到 Injective Testnet"

# 1. 编译合约
echo "📦 编译 CosmWasm 合约..."
cd contracts/ai-agent
cargo wasm
cd ../..

# 2. 上传合约
echo "📤 上传合约到链上..."
STORE_TX=$(injectived tx wasm store artifacts/ai_agent.wasm \
    --from deployer \
    --chain-id injective-888 \
    --gas-prices 500000000inj \
    --gas auto \
    --gas-adjustment 1.3 \
    --output json)

CONTRACT_CODE_ID=$(echo $STORE_TX | jq '.logs[0].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')

echo "✅ 合约代码ID: $CONTRACT_CODE_ID"

# 3. 实例化合约
echo "🎮 实例化合约..."
INSTANTIATE_TX=$(injectived tx wasm instantiate $CONTRACT_CODE_ID \
    '{}' \
    --from deployer \
    --label "AI Game World" \
    --admin $(injectived keys show deployer -a) \
    --chain-id injective-888 \
    --gas-prices 500000000inj \
    --gas auto \
    --gas-adjustment 1.3 \
    --output json)

CONTRACT_ADDRESS=$(echo $INSTANTIATE_TX | jq '.logs[0].events[] | select(.type=="instantiate") | .attributes[] | select(.key=="_contract_address") | .value')

echo "✅ 合约地址: $CONTRACT_ADDRESS"

# 4. 部署 AI Agent SDK
echo "🤖 部署 AI Agent SDK..."
cd agent-sdk
npm install
npm run build

# 5. 配置环境变量
cat > .env << EOF
AI_CONTRACT_ADDRESS=$CONTRACT_ADDRESS
INJECTIVE_RPC=https://testnet.sentry.injective.network
OA_PRIVATE_KEY=$OA_PRIVATE_KEY
OPENAI_API_KEY=$OPENAI_API_KEY
EOF

echo "✅ 部署完成！"
echo "📝 合约地址: $CONTRACT_ADDRESS"
echo "🤖 启动 AI Agent: npm run start-agent"