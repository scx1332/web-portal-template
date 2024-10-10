
import { formatEther } from "ethers/lib/utils";
import { BigNumber } from "bignumber.js";
export interface BlockFromApi {
    address: string;
    blockNumber: number;
    timestamp: string;
    balance: string;
    balanceDiff: string;
    updated: string;
    blockMiner: string;
    consensusReward: string;
    mevReward: string;
    blockReward: string;
    amountIncoming: string;
    amountOutgoing: string;
}

export interface BlocksSummary {
    totalEntries: number;
    totalDiff: bigint;
    totalSumDiff: bigint;
    totalConsensusReward: bigint;
    totalMevReward: bigint;
    totalBlockReward: bigint;
    totalAmountIncoming: bigint;
    totalAmountOutgoing: bigint;
}


export function analyze_blocks(blocks: BlockFromApi[]) {
    const totalEntries = blocks.length;

    const totalDiff = blocks.length > 1 ? BigInt(blocks[blocks.length - 1].balance) - BigInt(blocks[0].balance) : 0;
    let totalSumDiff = BigInt(0);
    let totalConsensusReward = BigInt(0);
    let totalMevReward = BigInt(0);
    let totalBlockReward = BigInt(0);
    let totalAmountIncoming = BigInt(0);
    let totalAmountOutgoing = BigInt(0);
    for (let i = 1; i < blocks.length; i++) {
        totalSumDiff = totalSumDiff + BigInt(blocks[i].balanceDiff);
        totalConsensusReward = totalConsensusReward + BigInt(blocks[i].consensusReward);
        totalMevReward = totalMevReward + BigInt(blocks[i].mevReward);
        totalBlockReward = totalBlockReward + BigInt(blocks[i].blockReward);
        totalAmountIncoming = totalAmountIncoming + BigInt(blocks[i].amountIncoming);
        totalAmountOutgoing = totalAmountOutgoing + BigInt(blocks[i].amountOutgoing);
    }

    return {
        totalEntries: totalEntries,
        totalDiff: totalDiff,
        totalSumDiff: totalSumDiff,
        totalConsensusReward: totalConsensusReward,
        totalMevReward: totalMevReward,
        totalBlockReward: totalBlockReward,
        totalAmountIncoming: totalAmountIncoming,
        totalAmountOutgoing: totalAmountOutgoing,
    } as BlocksSummary;
}