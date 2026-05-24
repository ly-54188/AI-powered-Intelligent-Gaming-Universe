import { InjectiveOAOClient } from '@injective/oao-sdk';
import { VerifiableAIModel } from '@openledger/verifiable-ai';

export class OpenLedger {
    private oaoClient: InjectiveOAOClient;
    private verifiableModel: VerifiableAIModel;

    constructor(privateKey: string) {
        this.oaoClient = new InjectiveOAOClient({
            privateKey,
            network: 'testnet',
        });

        this.verifiableModel = new VerifiableAIModel({
            modelId: 'gpt-4-verifiable',
            attestationType: 'zk-proof',
        });
    }

    async generateProof(input: OAInput): Promise<ORAProof> {
        // 1. 生成推理证明
        const inference = await this.verifiableModel.infer({
            input: input.input,
            metadata: {
                modelId: input.modelId,
                timestamp: Date.now(),
                agentId: input.agentId,
            },
        });

        // 2. 提交到OAO链上验证
        const verification = await this.oaoClient.submitInference({
            inputHash: this.hashInput(input.input),
            output: inference.output,
            proof: inference.proof,
            modelId: input.modelId,
        });

        // 3. 等待链上确认
        await verification.waitForConfirmation();

        return {
            txHash: verification.txHash,
            proof: inference.proof,
            inputHash: this.hashInput(input.input),
            output: inference.output,
            validator: verification.validator,
        };
    }

    async verifyOnChain(proof: ORAProof): Promise<boolean> {
        return await this.oaoClient.verifyInference({
            txHash: proof.txHash,
            expectedOutput: proof.output,
        });
    }

    private hashInput(input: string): string {
        const crypto = require('crypto');
        return crypto.createHash('sha256').update(input).digest('hex');
    }
}