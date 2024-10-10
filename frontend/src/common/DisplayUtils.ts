import {formatEther} from "ethers/lib/utils";
import {BigNumber} from "bignumber.js";

export const displayEth = (balance: bigint | string, decimals: number) => {
    if (decimals == 18) {
        return formatEther(BigInt(balance));
    }
    return BigNumber(formatEther(BigInt(balance))).toFixed(decimals);
};
