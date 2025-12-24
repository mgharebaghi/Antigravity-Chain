export const ONE_AGT = 1000000;

export const formatNumber = (num: number | string | undefined | null, scale = true) => {
    if (num === undefined || num === null) return "0";
    const n = typeof num === "string" ? parseFloat(num) : num;
    if (isNaN(n)) return "0";

    // Convert atomic units to AGT if scale is true
    const value = scale ? n / ONE_AGT : n;

    return new Intl.NumberFormat('en-US', {
        minimumFractionDigits: 0,
        maximumFractionDigits: scale ? 6 : 0
    }).format(value);
};

export const parseAmount = (val: string): number => {
    const floatVal = parseFloat(val);
    if (isNaN(floatVal)) return 0;
    return Math.round(floatVal * ONE_AGT);
};

export const calculateFee = (amount: number) => {
    const fee = Math.ceil(amount * 0.0001);
    // Minimum fee 0.001 AGT
    return Math.max(1000, fee);
};
