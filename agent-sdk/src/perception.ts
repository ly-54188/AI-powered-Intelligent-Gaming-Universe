import { ChainGrpcBankApi, ChainGrpcOracleApi, ChainGrpcExchangeApi } from '@injectivelabs/sdk-ts';
import { IndexerGrpcSpotApi, IndexerGrpcDerivativesApi } from '@injectivelabs/indexer-grpc';

export class PerceptionSystem {
    private bankApi: ChainGrpcBankApi;
    private oracleApi: ChainGrpcOracleApi;
    private spotApi: IndexerGrpcSpotApi;
    private derivApi: IndexerGrpcDerivativesApi;

    constructor(endpoints: InjectiveEndpoints) {
        this.bankApi = new ChainGrpcBankApi(endpoints.chainGrpc);
        this.oracleApi = new ChainGrpcOracleApi(endpoints.chainGrpc);
        this.spotApi = new IndexerGrpcSpotApi(endpoints.indexerGrpc);
        this.derivApi = new IndexerGrpcDerivativesApi(endpoints.indexerGrpc);
    }

    async perceive(): Promise<WorldState> {
        // 并行获取所有感知数据
        const [
            marketPrice,
            orderBook,
            liquidityPools,
            recentTrades,
            oraclePrices,
            onChainEvents,
            socialSentiment,
        ] = await Promise.all([
            this.getMarketPrice('inj/usdt'),
            this.getOrderBook('inj/usdt'),
            this.getLiquidityPools(),
            this.getRecentTrades('inj/usdt', 100),
            this.getOraclePrices(['INJ', 'ETH', 'BTC']),
            this.getRecentEvents(),
            this.getSocialSentiment('injective'),
        ]);

        // 分析市场情绪
        const sentiment = this.analyzeSentiment({
            price: marketPrice,
            volume: orderBook.totalVolume,
            trades: recentTrades,
            social: socialSentiment,
        });

        // 检测异常事件
        const events = this.detectEvents({
            price: marketPrice,
            oraclePrices,
            liquidity: liquidityPools,
        });

        return {
            timestamp: Date.now(),
            marketPrice,
            orderBook,
            liquidityPools,
            recentTrades,
            oraclePrices,
            sentiment,
            events,
        };
    }

    private async getMarketPrice(marketId: string): Promise<PriceData> {
        const market = await this.spotApi.fetchMarket(marketId);
        const price = await this.spotApi.fetchSpotPrice(marketId);

        return {
            marketId,
            price: price.price,
            change24h: price.change24h,
            volume24h: price.volume24h,
        };
    }

    private async getOrderBook(marketId: string): Promise<OrderBook> {
        const orderbook = await this.spotApi.fetchOrderbook(marketId);

        return {
            buys: orderbook.buys.slice(0, 10),
            sells: orderbook.sells.slice(0, 10),
            spread: orderbook.sells[0]?.price - orderbook.buys[0]?.price,
            totalVolume: this.calculateTotalVolume(orderbook),
        };
    }

    private async getRecentEvents(): Promise<OnChainEvent[]> {
        // 查询最近的AI行动事件
        const contractEvents = await this.queryContractEvents(
            process.env.AI_CONTRACT_ADDRESS!,
            100
        );

        return contractEvents.map(event => ({
            type: event.type,
            agentId: event.agentId,
            data: event.data,
            timestamp: event.timestamp,
        }));
    }

    private analyzeSentiment(data: SentimentData): Sentiment {
        // 综合多维度计算情绪指数
        let score = 50; // 中性

        // 价格变化影响
        if (data.price.change24h > 5) score += 10;
        if (data.price.change24h < -5) score -= 10;

        // 交易量影响
        if (data.volume > 1e6) score += 5;

        // 社交媒体情绪
        score += data.social.sentiment * 10;

        return {
            score: Math.min(100, Math.max(0, score)),
            label: this.getSentimentLabel(score),
            fearGreedIndex: this.calculateFearGreed(score),
        };
    }

    private detectEvents(data: MarketData): WorldEvent[] {
        const events: WorldEvent[] = [];

        // 检测价格剧烈波动
        if (Math.abs(data.price.change24h) > 10) {
            events.push({
                type: 'volatility_spike',
                severity: 'high',
                description: `价格波动 ${data.price.change24h}%`,
            });
        }

        // 检测流动性危机
        if (data.liquidity.totalValue < 100000) {
            events.push({
                type: 'liquidity_crisis',
                severity: 'critical',
                description: '流动性池枯竭',
            });
        }

        // 检测预言机异常
        if (this.detectOracleAnomaly(data.oraclePrices)) {
            events.push({
                type: 'oracle_anomaly',
                severity: 'medium',
                description: '预言机价格偏离',
            });
        }

        return events;
    }
}