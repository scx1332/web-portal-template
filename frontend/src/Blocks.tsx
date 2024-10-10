import React, { useEffect } from "react";

import { useLoginOrNull } from "./LoginProvider";
import { backendFetch } from "./common/BackendCall";
import { formatEther } from "ethers/lib/utils";
import { BigNumber } from "bignumber.js";
import {analyze_blocks, BlockFromApi} from "./logic/Accounting";
import {displayEth} from "./common/DisplayUtils";

interface Scan {
    address: string;
    firstBlocNumber: number;
    firstBlockTimestamp: string;
    nextBlockNumber: number;
    nextBlockTimestamp: string;
}

const Blocks = () => {
    const loginInformation = useLoginOrNull();

    const [address, setAddress] = React.useState<string | null>(null);

    const [currency, setCurrency] = React.useState("ETH");
    const [blocks, setBlocks] = React.useState<Array<BlockFromApi>>([]);
    const [loading, setLoading] = React.useState(false);
    const [scans, setScans] = React.useState<Array<Scan>>([]);
    const getScans = async () => {
        setLoading(true);
        const response = await backendFetch("/api/scan/all", {
            method: "Get",
        });
        const data = await response.json();
        setScans(data);
        if (data.length > 0) {
            setAddress(data[0].address);
        }
        setLoading(false);
    };
    const getBlocks = async () => {
        setBlocks([]);
        setLoading(true);
        const response = await backendFetch(`/api/scan/${address}/blocks`, {
            method: "Get",
        });
        const data = await response.json();
        setBlocks(data);
        setLoading(false);
    };
    useEffect(() => {
        getScans().then();
    }, []);

    useEffect(() => {
        getBlocks().then();
    }, [address]);

    const toClass = (balance: string | bigint) => {
        if (BigInt(balance) > 0) {
            return "positive";
        } else {
            return "zero";
        }
    };

    const toUSD = BigInt(2469);
    const toEUR = BigInt(2218);
    const toPLN = BigInt(9588);

    const getDecimals = (currency: string) => {
        switch (currency) {
            case "ETH":
                return 5;
            case "USD":
                return 2;
            case "EUR":
                return 2;
            case "PLN":
                return 2;
            default:
                return 5;
        }
    };
    const toEth = (balance: string | bigint, decimals: number) => {
        let convertionRate = BigInt(1);
        if (currency == "USD") {
            convertionRate = toUSD;
        }
        if (currency == "EUR") {
            convertionRate = toEUR;
        }
        if (currency == "PLN") {
            convertionRate = toPLN;
        }
        return displayEth(BigInt(balance) * convertionRate, decimals);
    };


    const DisplayEther = (props: { balance: string | bigint }) => {
        return (
            <span
                className={toClass(props.balance)}
                title={toEth(props.balance, 18) + " Ether = " + props.balance + " Wei"}
            >
                {toEth(props.balance, getDecimals(currency))}
            </span>
        );
    };



    let summary = analyze_blocks(blocks);

    for (let i = 0; i < blocks.length; i++) {
        const blockBalanceRight =
            BigInt(blocks[i].consensusReward) +
            BigInt(blocks[i].mevReward) +
            BigInt(blocks[i].blockReward) +
            BigInt(blocks[i].amountIncoming) -
            BigInt(blocks[i].amountOutgoing);
        const blockBalanceLeft = BigInt(blocks[i].balanceDiff);

        if (blockBalanceLeft != blockBalanceRight) {
            if (blocks[i].blockMiner == address) {
                blocks[i].blockReward = (blockBalanceLeft - blockBalanceRight).toString();
                blocks[i].mevReward = blocks[i].amountIncoming;
                blocks[i].amountIncoming = "0";
            } else {
                console.log(
                    "Balance mismatch at block",
                    blocks[i].blockNumber,
                    "diff=",
                    formatEther(blockBalanceLeft),
                    "right=",
                    formatEther(blockBalanceRight),
                );
            }
        }
    }
    const renderBlock = (idx: number, block: BlockFromApi) => {
        return (
            <tr key={block.blockNumber}>
                <td>{idx}</td>
                <td>{block.blockNumber}</td>
                <td>{block.timestamp}</td>
                <td>
                    <DisplayEther balance={block.balance} />
                </td>
                <td>
                    <DisplayEther balance={block.balanceDiff} />
                </td>
                <td>
                    <DisplayEther balance={block.consensusReward} />
                </td>
                <td>
                    <DisplayEther balance={block.mevReward} />
                </td>
                <td>
                    <DisplayEther balance={block.blockReward} />
                </td>
                <td>
                    <DisplayEther balance={block.amountIncoming} />
                </td>
                <td>
                    <DisplayEther balance={block.amountOutgoing} />
                </td>
            </tr>
        );
    };

    if (loading) {
        return <div>Loading...</div>;
    }
    if (address == null) {
        return <div>Address not selected</div>;
    }
    return (
        <div>
            <div style={{ padding: 10 }}>
                Select address:
                <select onChange={(e) => setAddress(e.target.value)}>
                    {scans.map((scan, idx) => {
                        return (
                            <option key={idx} selected={address == scan.address} value={scan.address}>
                                {scan.address}
                            </option>
                        );
                    })}
                </select>
            </div>
            <div style={{ padding: 10 }}>
                Select currency:
                <select onChange={(e) => setCurrency(e.target.value)}>
                    <option selected={currency == "ETH"} value={"ETH"}>
                        ETH
                    </option>
                    <option selected={currency == "USD"} value={"USD"}>
                        USD
                    </option>
                    <option selected={currency == "EUR"} value={"EUR"}>
                        EUR
                    </option>
                    <option selected={currency == "PLN"} value={"PLN"}>
                        PLN
                    </option>
                </select>
            </div>
            Blocks
            <div className={"block-table-header"}>
                <div style={{ left: 0 }}>No</div>
                <div style={{ left: 100 }}>Block number</div>
                <div style={{ left: 200 }}>Timestamp</div>
                <div style={{ left: 300 }}>Balance</div>
                <div style={{ left: 400 }}>Balance Diff</div>
                <div style={{ left: 500 }}>Consensus Reward</div>
                <div style={{ left: 600 }}>MEV Reward</div>
                <div style={{ left: 700 }}>Block Reward</div>
                <div style={{ left: 800 }}>Amount Incoming</div>
                <div style={{ left: 900 }}>Amount Outgoing</div>
            </div>
            <div style={{ height: 300, overflow: "auto" }}>
                <table className={"block-table"}>
                    <tbody>
                        {blocks.map((block, idx) => {
                            return renderBlock(idx, block);
                        })}
                    </tbody>
                </table>
            </div>
            <div className={"block-table-footer"}>
                <div style={{ left: 0 }}>Total count</div>
                <div style={{ left: 100 }}>-</div>
                <div style={{ left: 200 }}>-</div>
                <div style={{ left: 300 }}>Balance</div>
                <div style={{ left: 400 }}>Total balance diff</div>
                <div style={{ left: 500 }}>Total consensus reward</div>
                <div style={{ left: 600 }}>Total MEV Reward</div>
                <div style={{ left: 700 }}>Total block reward</div>
                <div style={{ left: 800 }}>Total amount incoming</div>
                <div style={{ left: 900 }}>Total amount outgoing</div>
            </div>
            <table className={"block-table"}>
                <tr>
                    <td>{summary.totalEntries}</td>
                    <td></td>
                    <td></td>
                    <td>
                        <DisplayEther balance={summary.totalDiff} />
                    </td>
                    <td>
                        <DisplayEther balance={summary.totalSumDiff} />
                    </td>
                    <td>
                        <DisplayEther balance={summary.totalConsensusReward} />
                    </td>
                    <td>
                        <DisplayEther balance={summary.totalMevReward} />
                    </td>
                    <td>
                        <DisplayEther balance={summary.totalBlockReward} />
                    </td>
                    <td>
                        <DisplayEther balance={summary.totalAmountIncoming} />
                    </td>
                    <td>
                        <DisplayEther balance={summary.totalAmountOutgoing} />
                    </td>
                </tr>
            </table>
            <div>
                <h3>Checks</h3>
                <div>Difference Between last and first block:</div>
                <div>{summary.totalDiff.toString()} Wei</div>
                <div>Sum of changes</div>
                <div>{summary.totalSumDiff.toString()} Wei</div>

                <div>Sum of incoming txs</div>
                <div>
                    <DisplayEther
                        balance={(
                            summary.totalConsensusReward +
                            summary.totalMevReward +
                            summary.totalBlockReward +
                            summary.totalAmountIncoming
                        )}
                    />
                </div>
                <div>Sum of outgoing txs</div>
                <div>
                    <DisplayEther balance={summary.totalAmountOutgoing} />
                </div>
                <div>Balance sum</div>
                <div>
                    <DisplayEther
                        balance={
                            summary.totalConsensusReward +
                            summary.totalMevReward +
                            summary.totalBlockReward +
                            summary.totalAmountIncoming -
                            summary.totalAmountOutgoing
                        }
                    />
                </div>
            </div>
        </div>
    );
};

export default Blocks;
