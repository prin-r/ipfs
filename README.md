# Estimating Full Transaction Gas From Calldata Size

This document explains the **Foundry** test design, the **polynomial models** (linear / quadratic / cubic), the **Monte-Carlo** validation, and the **conclusions** for the on-chain environment.

> **Goal.** Let a relayer with *minimal native balance* relay forever by paying the **entire transaction gas** from the **consumer’s vault**.
> We split a relay tx into two parts:
>
> ```
> TotalTxGasUsed  =  baseGas(x)  +  targetGasUsed
>                    ^              ^
>                    |              └─ measured consumer-side work (easy)
>                    └─ router + EVM overhead, a function of calldata length x
> ```
>
> * `targetGasUsed` = the “consumer side” gas, which is reimbursed to the relayer from the vault.
> * `baseGas(x)`    = everything else (intrinsic base, calldata pricing, memory expansion, cold/warm penalties, router overhead).
> * We **measure** `targetGasUsed` and **learn** `baseGas(x)` from data.

---

## Contents

* [Overview](#overview)
* [Intuition And Hypothesis](#intuition-and-hypothesis)
* [Foundry Test](#foundry-test)
* [On-Chain](#on-chain)
* [Modeling $f(x)$](#modeling-fx)
* [Results (Typical)](#results-typical)
* [Putting It Into Production](#putting-it-into-production)
* [Results](#figures--placeholders)
* [Transaction ](#transaction-placeholders)
* [Key Code Snippets](#key-code-snippets)
* [Appendix: EVM Cost Anatomy (Plain-Text)](#appendix-evm-cost-anatomy-plain-text)
* [Summary](#summary)

---

## Overview

We decompose the tx gas:

* **`targetGasUsed`** : **consumer-side** cost (settled from the vault). Easy to read on-chain (event) or measure in Foundry with `Δgasleft()` and `Δbalance`.
* **`baseGas(x)`** &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;: **router + EVM** overhead that depends mainly on **calldata length** `x`.

We fit **linear**, **quadratic**, and **cubic** polynomials of `x` and pick the best via **Monte-Carlo cross-validation** (on Sepolia/hoodi receipts). In practice, **quadratic** wins: linear underfits; cubic often overfits.

---

## Intuition And Hypothesis

It’s useful to sketch **plausible shapes** for `baseGas(x)` (the portion of total gas not accounted for by the consumer’s own work).

### 1) Strong linearity

Post-Istanbul, calldata is priced per byte: roughly **16 gas per non-zero byte** and **4 gas per zero byte**. If the relay payload grows proportionally with the number of signals, this induces a **dominant linear trend** in `x = calldata.length`.

### 2) Subtle quadraticity

The EVM charges for growing the transient memory footprint. A plain-text rendition of the memory cost model (see [Yellow Paper](https://ethereum.github.io/yellowpaper/paper.pdf)):

```
C_mem(a) = G_memory * a + floor(a^2 / 512)
DeltaC   = C_mem(a1) - C_mem(a0)
```

where `a` is the size in 32-byte words. The `a^2/512` term can introduce **gentle curvature** (sub-quadratic over typical ranges) when decoding, copying, or aggregating arrays derived from calldata. In many real paths, this quadratic component is **present but small** relative to the linear calldata term.

### 3) The higher-order effects (e.g., cubic) may appear

In principle, it is possible if memory expansion compounds in a way that scales super-linearly with payload (e.g., nested growth, repeated re-allocations tied to `x`). In practice we expect this to be **weak**, and any apparent cubic fit could be a symptom of **overfitting** rather than a true system property.

**Modeling stance (to be tested empirically):**

* Start with **linear** as a baseline (captures calldata pricing).
* Allow **quadratic** to capture mild curvature from memory expansion.
* Treat **cubic** with caution: higher degrees can reduce in-sample error but often **overfit** and generalize worse to unseen payload sizes.

The Monte-Carlo cross-validation in later sections evaluates these hypotheses against real receipts.

---

## Methodology

To empirically determine the optimal model for `baseGas(x)`, we follow a four-step process. The goal is to isolate the router and EVM overhead as a function of calldata length `x`, which we assume to be a polynomial.

### 1) Data Preparation

First, we construct a dataset of `(x, baseGas)` pairs for each target network (e.g., Sepolia, Hoodi).

  - **`x`**: The `calldata` size in bytes for each relayed transaction.
  - **`baseGas`**: The calculated overhead, derived from on-chain receipts as `receipt.gasUsed − targetGasUsed`.

### 2) Baseline Model Fitting

To gain an initial understanding of the data's structure, we perform a single split of the dataset into training and validation sets. We then fit linear, quadratic, and cubic polynomials to the training data.

This step provides a quick sanity check of the model coefficients and reveals the dominant trends.

**An Example Implementation:**

```python
import numpy as np

# Sample dataset of (calldata_length, base_gas) pairs
xy = [[356, 96455],[420, 97561],[484, 98655],[548, 99737],[612, 100807],[676, 101901],[740, 102983],[804, 104066],[868, 105148],[932, 106231],[996, 107313],
# ... (data continues)
]

calldata_len = [xx for xx, _ in xy]
base_gas     = [yy for _, yy in xy]

# NOTE: This 50/50 split is arbitrary.
split = int(len(xy) * 0.5)
train_x = np.array(calldata_len[:split])
train_y = np.array(base_gas[:split])
val_x   = np.array(calldata_len[split:])
val_y   = np.array(base_gas[split:])

# Fit
lin  = np.polyfit(train_x, train_y, 1)
quad = np.polyfit(train_x, train_y, 2)
cube = np.polyfit(train_x, train_y, 3)

print("--- Optimal Linear ---")
print(f"y = a*x + b    | a≈{lin[0]}  b≈{lin[1]}")
print("\n--- Optimal Quadratic ---")
print(f"y = a*x^2+bx+c | a≈{quad[0]} b≈{quad[1]} c≈{quad[2]}")
print("\n--- Optimal Cubic ---")
print(f"y = ax^3+bx^2+cx+d | a≈{cube[0]} b≈{cube[1]} c≈{cube[2]} d≈{cube[3]}")
```

**Example Output:**
```
--- Optimal Linear ---
y = a*x + b    | a≈16.937874399759874  b≈90434.389654862

--- Optimal Quadratic ---
y = a*x^2+bx+c | a≈1.2163350165786751e-05 b≈16.89106982832197 c≈90469.04041220056

--- Optimal Cubic ---
y = ax^3+bx^2+cx+d | a≈-6.92054848651535e-10 b≈1.6157890752213517e-05 c≈16.88444633635134 d≈90471.92608110276
```

This output shows a dominant linear relationship of ~16 gas/byte, with a small positive quadratic term consistent with memory expansion costs.

The negative cubic term suggests potential overfitting. This is logically inconsistent because `baseGas(x)` must be a monotonically increasing function, as it is impossible for gas costs to decrease as calldata size increases.

### 3) Model Selection via Monte Carlo Cross-Validation

A single arbitrary data split can be misleading or inaccurate. To robustly select the best model, we use Monte Carlo cross-validation:

1.  Repeat the process of randomly splitting the data into training and validation sets thousands of times (e.g., 5,000-10,000 trials).
2.  For each trial, fit all three polynomial degrees and calculate their validation error (e.g., Root Mean Square Error - RMSE).
3.  Aggregate the error metrics (mean and standard deviation) for each model degree across all trials.

The winning model is the one with the lowest average RMSE. As a heuristic (the "one standard deviation rule"), if a simpler model's mean RMSE is within one standard deviation of a more complex model's, the simpler model is preferred to avoid overfitting. We also analyze model stability by varying the training set size (e.g., from 10% to 90%).

---

## Data Collection

We gather `(x, baseGas)` from **two sources**:

### 1) Foundry (repeatable, lab-like)

**A Single Measurement**
 1. Build the message `relay(bytes,address,uint256)` for the given **N** signals.
 2. Measure:
    * `relayGas = Δgasleft()` (gas spent by `tunnelRouter.relay(...)` in the test),
    * `targetGasUsed = Δrelayer.balance` (with `gasPrice=1`, `additionalGas=0` which equals the consumer-side work).
 3. Compute a **Foundry** baseline:
    ```
        baseGas_foundry = relayGas − targetGasUsed
    ```
    just for **sanity checks** and **router calibration**, but **chain-level** `baseGas(x)` comes from **real receipts**:

    ```
        baseGas_chain = tx.gasUsed − targetGasUsed
    ```

**For each trial**

 1. Snapshot/revert per trial to reduce hot-storage bias leaking across measurements:
    ```solidity
        uint256 snap = vm.snapshot();
        // single measurement here
        vm.revertTo(snap);
    ```

 2. Reset the free memory pointer before each relay to reduce memory accumulation:
    ```solidity
        assembly { mstore(0x40, 0x80) }
    ```

 3. Validate that relay effects are correct (prices/timestamps updated, sequence advanced, etc.).

**Calibrating the router**

  * Fit a quadratic $c_2x^2 + c_1x + c_0$ to the `baseGas(x)` where `x = calldataLen`,
  * Pack coefficients (e.g., 3×80-bit) and call `tunnelRouter.setAdditionalGasUsed(...)`,
  * Verify the **residual gap** between compensated router payout and measured gas shrinks (we assert a small bound, e.g., < \~50 gas).

Foundry data is great for router calibration and sanity checks, but our production coefficients are ultimately fit to **on-chain receipts** below.

### 2) On-chain receipts

**For each real relay tx**

1. `total = receipt.gasUsed` → TotalTxGasUsed
2. `target = withdrawnAmount` (from router event) → targetGasUsed
3. `x = len(tx.input)` → calldata length
4. Calculate
    ```
        baseGas = total − target
    ```
5. Append `(x, baseGas)` to your dataset (per network).

Collect a dataset of pairs `(x, baseGas)` per chain (Sepolia / hoodi / etc).

<details> <summary><code>Python3</code> — click to expand</summary>

```python
import json
from typing import Any, Dict, List, Tuple
from web3 import Web3

# --- Config ---
with open("./txs.json", "r") as f:
    CONFIG = json.load(f)

CHAIN = "hoodi"  # or "sepolia"
TX_HASHES: List[str] = CONFIG[CHAIN]["txs"]
RPC_ENDPOINTS: List[str] = CONFIG[CHAIN]["rpcs"]

# Normalize to lowercase, no "0x"
TARGET_TOPIC = (
    "a623627c34311a71b36da152b44076ede51861d701868671c9a6dfdd0f5da00a".lower()
)
TARGET_EVENT = (
    "04eda370f8b8612fa7266d7ebbd41af9d694e19793fe9d9ff31b3ddbd99b08e1".lower()
)


def _norm_hex(s: str) -> str:
    """Lowercase, strip 0x."""
    return s[2:].lower() if s.startswith("0x") else s.lower()


def _extract(receipt_logs: List[Dict[str, Any]]) -> Tuple:
    """Return (topic_hit_count, [withdrawnAmounts]) from matching logs."""
    count, ev_vals = 0, []
    for lg in receipt_logs:
        topics = lg.get("topics") or []
        if topics:
            t0 = _norm_hex(topics[0].hex())
            if t0 == TARGET_TOPIC:
                count += 1
            if t0 == TARGET_EVENT:
                data_hex = _norm_hex(lg.get("data", b"").hex())
                # first 32 bytes = 64 hex chars
                if len(data_hex) >= 64:
                    ev_vals.append(int(data_hex[:64], 16))
    return count, ev_vals


def main() -> None:
    w3 = Web3(Web3.HTTPProvider(RPC_ENDPOINTS[0]))
    if not w3.is_connected():
        raise RuntimeError(f"Failed to connect to RPC: {RPC_ENDPOINTS[0]}")
    print(f"Connected: {RPC_ENDPOINTS[0]}")

    base_gases: List[int] = []

    # (calldata_len_bytes, base_gas)
    pairs: List[tuple[int, int]] = []

    for i, txh in enumerate(TX_HASHES):
        try:
            rcpt = w3.eth.get_transaction_receipt(txh)
            if rcpt is None:
                print(f"[{i}] {txh[:10]}… not mined; skip")
                continue

            tx = w3.eth.get_transaction(txh)
            calldata_len_bytes = len(tx["input"])

            topics_count, event_vals = _extract(rcpt["logs"])
            if not event_vals:
                print(
                    f"[{i}] {txh[:10]}… gas_used={rcpt['gasUsed']} target_gas=NA base_gas=NA calldata_len={calldata_len_bytes}B topics_count={topics_count}"
                )
                continue

            target_gas = event_vals[0]
            gas_used = rcpt["gasUsed"]
            base = gas_used - target_gas

            base_gases.append(base)
            pairs.append([calldata_len_bytes, base])

            # One line per tx
            print(
                f"[{i}] {txh[:10]}… gas_used={gas_used} target_gas={target_gas} base_gas={base} calldata_len={calldata_len_bytes}B topics_count={topics_count}"
            )

        except Exception as e:
            print(f"[{i}] {txh[:10]}… ERROR: {e}")

    # A summary for downstream use
    print("pairs(len(calldata_bytes), base_gas) =", pairs)


if __name__ == "__main__":
    main()
```

</details>


<details> <summary><code>txs.json</code> — click to expand</summary>

```json
{
    "sepolia": {
        "rpcs": ["https://1rpc.io/sepolia"],
        "txs": [
            "43ce582ec8478115538a7a003518ccb591817bd07176c41f494f688411defae5",
            "0c50665f98d890b24d3955e7d5f2bc7f5616145e2227c685c4342000192e9c98",
            "03d7df32659e7c945a24871007aab8a6ae3e2ae14323e9f8648d2a23a66e0cb9",
            "112edbaf226e8490edc4234a3ed1572bf83bf33960b5e9b7a8645b4b1ef65b42",
            "fa91e48eaf15d1765235e277e1eb21610a46a4c26a6551c60ccb39df86a09afc",
            "c010c266bb91041d4242307f6654576d7f5cec5d9a580d079c41353d687bf835",
            "129a01590e04f68adc9873635006874a7131896903f6f8ccc14be450a92e519c",
            "cb33ca6baf085571786b4350c0e63096ac5abfa57e5a5f411800dd42d493eb09",
            "a2af520533edd84bcd38d39891800b31651c61a89fe2840ab103f8a424a3eb23",
            "b654eab020fff033ae1bd7b9fdf00eb2060b85eed6ab2a9eadd21a590d2d9272",
            "b4cdc9d421c47c8611405ee9c24959635e34c3cd0b170f42ef1f260ad3c093a9",
            "bfba0cf9bb6b2d971739c8c83c5b9ea3b430e6f9e014e3453033fc3c5a617f96",
            "8e6cae766dbf13fd3c859054d2de3aa03cffd0e8f32edb8c4417fb0c052fcd9c",
            "04d3211c81c89e8cbb2617f8f659695e67a45c5a8d04c7021c801b2952b474df",
            "7a2566c151ea73ffc972f3f33be8171a3ef57f01ffc128d0884b1028143d9e79",
            "bcc42da5f86d2e61ff515d72bc20d21c7d4fe06f45e8b101191df3ed9d42a4db",
            "c9d84b5d6a1d52ffbe202753386f026cbf0f0ae156f08d945e285703382934f9",
            "2634437b33b8affdabc85b60ba963d0e5000737bf584a52de438090115d3fd52",
            "a315d7b6988b2250f56bd23897278482d635a54dff747858ef50389cb85ec66e",
            "9d546e769b725deefa6d28e2daddc1e02f6e3a6939640b80df869ce22fa45eb9",
            "953c29b084fb526ca2cffc98114ac831d1cde00212556291afeec9be1331fbaf",
            "636949e39a96639ff1ee05ad9c12909874fb239bfc7429a0f6c255628fe1c5ea",
            "1516dfed21245ba99f1653f7fb64d8d29f0d872cabbcf09976283e77248f96c4",
            "b2a10dc1b6a7b4fb96b1c15cdcada07ad70e9c7b7fe761eb12aaee678fddc101",
            "d06a456df31778a662898c169e233d18296bf743597d794a221514c7db50d3ba",
            "7649a70b8c5dc9a85553a047219af6df2e42938af48c0341a132da6411ccabc7",
            "7453cde51c47643faaaeeff15432acf04a6e10ef9366301d0e91d4fca11be215",
            "e02ad47e8ca677b69b0746480b4802f144909dad35951262786f82e81fea387d",
            "4590c8c59e6ec69ed6ad8a92a7d5765f3f4a4dba31f6ff5583cc7cf7aeb6eabe",
            "8d78d2a7bdc0eb2edbc572c2b1c6d5d5713a8f96397fd300b53cc89f318bb7b4",
            "4e6cda60881a40883d8ceed720c808feb32baf84e09a73ae4ee1f1a78a231cd2",
            "fdef2e282813eaec5bfd746c1b3815b61068a244685ca1f1b89de7c640d4ea49",
            "afb8884f5825ddf5f3141e8da5ddb076d568957e921e27ad3279c88a09812bc2",
            "962bcd9c4c2c37624f65886ff5846bc3ac832eff4509e2f383c5669bf8d98600",
            "e4f11cefe2a74bb4c7d9bfa9ca63bf9d9cd9cec2d8e5ea9f43fc569ab5ebf1be",
            "36183653fe8b950f3079ec46accc0d971f9370be31fc0b75467260ac1c3da1a1",
            "6cf5dd29029431eb86359d3a893b7585dfeb271df55521a4216e00a8f448f402",
            "3a2e909a80b06dbf5f9d7518da4e1865fc3bc6b0c54118cbc52c7a40743d20ae",
            "c61584452ead299b4571255afcb279b4bba0d1bc6a9791051bcf50a55c5f9de5",
            "ef71aae22870ebded4fdc28fdb79dc09e86ab95b71da853060d99d6295dca8f5",
            "4495e9f72d074b12970426f9a4caa1085131953a114e770d33b1f35c3bb9c0c4",
            "057d0d1dbc3e6ccae118f6d2454344ba413b082ff073950567ea2db011614ea7",
            "a6b1ecc10e6cc3053c27311ad2596e637bbccd727e7142c1def23afacba5f91f",
            "3666d80bbcd829aa0e9e91c8e2dbf8a2cde535f8c86dac169d2ae3d1a0c43caf",
            "be555d06e340fa5146ef20aff13f5c1ead4517276044f7b30967b66a5db40753",
            "6c07850e1d2c795153929c144d9f07cefd038b5de789ca981189ca89026e07ca",
            "be9876bf6709798f48efe28ac4813a3003d75690cff980770b6c88b52d2d2a49",
            "a09a46694003eba6e4f3e5c628d1dcdade765c7fb04f4262ed930e87dcef9b6d",
            "55b190610a7eb63d7236ed7db1140690c14e82df0408608a11d5c20c24eb12c5",
            "3ae187750312de5f66bb731e5eafb00d4d7f5cd4a7d968868d27c592d1108b12",
            "d2907f11db975176d16d9739a21953c2f195ff0d6797da559c4bfe5f7ed34858",
            "b0cf27da8b67f1f4f46a98b20f1064fc3295595a31a63ec491bdbb759c870273",
            "9648a313b3e5e9162ccce1c1ee3a742072f707d67d211832933a18563ef71064",
            "36e3a6b9215ac83e801e25e11ce270f9d22e10cd2201d8ffc1bb463beae3f500",
            "9c9efd508417ff3d7a8c0f51e6e0990331947d9bc5acf1fcb459077e2f69fb95",
            "a1d3a209af270657cd95a561a23803d32490f26f2860a98a48bc89275bfe642c",
            "c6e94cbf60626d3f515d199dc2eb84ad8cf42db728d5fd3b0db840b1952b3830",
            "2a1c2fa0350be615aa478b3170c2afeb505d85014496aede6b2c303d83a4f583",
            "64c0af45f1846b9283f8fd9fadb64da82c19e225d57ebaa85041c57588d09cd5",
            "788015fd1c0e6f87418293658c1650cd8d0d4bde4a133014f9939a410df2242d",
            "ef2d0beac854a62af8871d6654c2747aa7c033d22551b6941c0211b34c7109c8",
            "373a1701ac47353a2cabc72653ea9d451a4a281477869fa267cf5e9f92f5a34f",
            "106168c84426384d7a940abb0436f9f7f35523bd133a3711b4b26fee791f9d48",
            "3d901a418f1635831320f6ab3b6d084815c62dcade8b0b6598ea3300b4d54ba3",
            "07b8c640c66e3d7ecd94af2b92ecc460901fff53763c800496f53d984a07f6ac",
            "1972ace99d6859981e81744191d895c80a73f6893acb90052a753ed06958518f",
            "154e5ad817e84625468ebf74f18bd765218b27ab4cc7d1064b1165097be886ee",
            "25445a986e27bdd2d0c4e141592c3c35e98629b3ce98cbee9a85bc89806c5f86",
            "1ca214b6735c55047d44f431c76b5b7b96a7506e77910518a38e38c199db0b4d",
            "7732d9c8bba75ecb15cb3afc45816d743857720cf77cde8615418b1d97982e83",
            "5564812dc14aada42d13cec940f8960eade66c75f13ce620498b1e8346d03761",
            "234c20c72f646547df8593afcdf47dc41e7bc69bd31ea4174816ec4ff83c846a",
            "6ceb3963831de91aefc6ced47e874270679337b63c52bad08e11ec3726ea91e1",
            "222faab5663371c6818ad6114c775c9ba77a238965f6dc039d6eb732ed2d1d39",
            "4e1194d40e02590b930187f91d5288a440c03ff93b83ff66cf781ee905390e70",
            "599f5c1110d92e15af8770a0d339e1a50328356dcdd4cf461f79d61989baaf89",
            "7662f8c0a73de5279894efb614b2fbaa96accb4fe1ae8454d3f398dde970ab11",
            "85166f25f10e76bb717ee848fe347b438382a181b2183cb05cee540e89b18173",
            "d7fa5bce9ea968856206902095b72630f867c03ea6b395009e14cb29b44506bb",
            "dcd3ef82dd680f582bc6887a28d9a9e9a5c1c8bee968bc947a5de2bf6ce2ff1c",
            "d04f46ef2838964b1fb485de9faf112435f080bc87790d1c74cf4c3739b31f2e",
            "b0971f86c9f41005548a036b4e43b3ef3316c2f0f30a72ff921a92bcfdfb57cf",
            "25820c594f1c0e0e990131f39bd31fd103c81f3fecbc3de7bda563c9b99a3184",
            "34123aeaca5ef071c2fc1488557c667eecc70e656fac390d657cf911399a67b4",
            "efc4bf1d5a302ecb57f84f7c91c0c7784fd8cbd2de2241ce9217c44183784bde",
            "7461eefce9ef4182c424372868ad73d7978af0071d24947d8838416ce1ddea1f",
            "00194c81663ff3f295bddfb4dfef4941da7434a16db04c0a2125e63dfac87cab",
            "2da9a8d1386219a9910935abe92dbf63dc1c2df95dbb9e891c8bee86112373d9",
            "8e6ff28480a7741c514f1d43dc78238ddb43042e6d1ff191685877ce591a591e",
            "d5437baacc48b2b4df88c9730f9ce75457abc5abe2d6838a5619fda5a6f33831",
            "4971b8a8c5d95cbcb38e24f6225196fd3396ddc9fcc2c2cdc7dfad1883f09860",
            "9f8173634d2d7a0f356575e13bb8591ad44249ea24eb2099bc522d4d757773eb",
            "79ae023134b5093506a45e5e2c2362005d8ac4d1ccfdf33952e79cdb6bdea450",
            "b9e0aaaea29e9746778d6ddc695efbc0da0ee13930f6269f2e960a2fc6cf2277",
            "064112b2b3692a08a17f590027f517497196d9dca795748f9dbced4cbb7577a6",
            "d5ad32186f0dbc2834014626c3c573c7ea3e52ce6378fc9a8e211da4a8ce07dd",
            "9716d0351fec75610be988bd39241377d5f1bbc1a7cd3d3b9e3121138e825659",
            "eb85b7345220360f5784f5863cf4d87a38d585e95f7169a0d6ae9a2fd5bfe2f8",
            "ca0ffe995e692a1e82b299fa7af751522721c11df128cec28e50dd69d9b1698e",
            "adac53bd0cc14098632783c3897c7841fccbf3702d0e8b0705e20e4757c4273c"
        ]
    },
    "hoodi": {
        "rpcs": ["https://1rpc.io/hoodi"],
        "txs": [
            "74ed9a64bc9dd453a5ac73834c33e81380837aa7874ae1791920fe37145b6102",
            "46460c22269448126420016395f1bd4ec805e23da0f0f78aaf9df2812946d8c9",
            "adb4d24bfe4ceca99b7c6de8f171d71ff2c94b7edfd8d02dc96cc717bd133eaa",
            "4f73c7fc1d83c5a7be4c5f94d582edbe285a67986dccd76252188508071d2169",
            "bfb2a21d3640ef2406bec21f52e392f2441c75aa2298318467c83e03a36f3030",
            "8c8b3d449c412c6c770a6df6da6966d0159c414dc1359fd8b095ea74ff802a5e",
            "74fc8908a94d088787f727521e0d1254af36e86109a4db7ac765b48070e17fb8",
            "28cc9ffcd362c1c11d0df277f161afe0e29f05d515c84ef5e7d347821f1f10f6",
            "caa6e38f54d46b53189d148b9c27cbb87b59395852aea5ee3ab8cde69c2a0d36",
            "226b9985a97ea33a73e324c73275695fa7af05b780a4837db751abc2fe2dba5f",
            "93341252d1e698ce7ca74ebdc84a45b7f060859b3d95bd9fb45de53ac3856347",
            "6741cb1ca4c15e835ffc360bd20f0a2781b5fee75450c43c3b5cb2c9fbf8b22e",
            "740368cb5b4b0b4cb1b56f3844eb2edea362affbe67f294132b9e0898b89379f",
            "14253b052d2f2cb9d56c91306b3f399fb555d7be772fd7a153ee2b35cd886b8c",
            "cc43ab48c6b8b0ac3496e23bff6eb30712ad9783e4c28772b32b49cb7e0a1ac8",
            "510c5d09c346a04878f1cad41bfefc0c4dbd889b1c1f48664dc842d5ccbc74b6",
            "1e22c8d8f2967f2af7c76543eb5ada1aef55d134e79f959d02a2d4b35d38d779",
            "647de8154d953fcdb93290cb07738279e040606adf3267e88c3cb2f1fef028c1",
            "e7eedeec50dc5470459ad4b95d7847f12aa6abfe82f48171a0e31281fc610de5",
            "0097fd2d9a926c3a157a17bdf941d2456967953bbdac295cd12bcb4b266f2cbf",
            "dfb73e99373e89380e547564c95ea3b6e049e5e27a2129cc2078cbcfa1c7a75c",
            "ef3e17ef88fe39c340cfd83e069a4df0f977c8524324c22ed2a6aa251bffc743",
            "b2208f2b01a322d7a9d5f2fd2d1f3357dea5f4a738f8fb5ed1bae5b4c48afb18",
            "ddb9ec8566fc8dd8f65428f79c2118014c99a2482cc1031a3706fc38ab873c3b",
            "9eea9772415a62fca19f343a4fa491f84d5a57802f4e7a9d3c9be14f7d7f4a53",
            "3cbe51ef87ff27bb1828bece482f51bf5b358de0a991cf0869c41723be518594",
            "34fea440347dfde6a669a8b6d5d7b484900c909b740c5e246b5cf5c68b402e39",
            "6125a094bb19a69fd63a12fa4a8473368299b9b0fee0742cadba7f6141803c9d",
            "b400ce52a09c82d46e32c09dc5e13015bb5ee985a2dcc60b1fa332349d35343b",
            "fee94283a70714a938e7e7f99976289714ea9f7b4de0ac08a5f4904c73813e03",
            "f8196032a2c68c7b07ec2042011f9b8fd35a0feda0b3e6a94626214f5bbfa6c5",
            "454e7a4d0641355d21ce2ce606fd59240564e42e205dd145a45ff96cf3640b5e",
            "849fd17c3e8e95d8f66bb6d92d5f8432504da171de30725f8150237fc3da65d8",
            "0641d3e8f52c7e48380d2e71e7d7485458678884cfc6812d08b47db4f6871f02",
            "63606a758e27dd58bc216cdd0d039eb0e211826ed16a8f03b5415c645872a4b1",
            "7645003e5db46daa438e2aeb36e1b38796777e1395f596a3df0a7e51edc11057",
            "796b5573bc1b5e3abbb3ee9b754a7b8b107fc6e8c9231cdcabb59b6d9da62e92",
            "11aad2ec1cdd0f6f163d7dd0789e93443703505923041860515726f60ca07f66",
            "4fe5c83caa32be94c4e1074b3272f1d05efb3082b29227bbafff656ddf26606e",
            "90950faf36d5ca106915a99ea6155d958409706d2643943e40ca7f294483134f",
            "8753799ebda8350b667c09777962abdf2edbebec8f0e40f83e770771d9f01598",
            "48eeb098975374b222f77b7b3264bf9961e51f37f0e9e5d81bfe08ae01e02837",
            "a30a084dc5ccbf2f01e3a166c6c694688590e37403ced7ef199eac94e390b2d0",
            "a4644c2f2bc01ddd3b2b3851d15f1610bed3a7a2b30d062ada8cca5fe2e35d5c",
            "4c94d3049431ae4cb48a564e4b297e1846edefe8698eaae83bd01de34c7fca2c",
            "c43a80e6529a0e4eab390e47260d2dce97804cc71ddfaeb1fd4929b23ef03d6f",
            "11e40009c1fbb5c2988f45beff13087fe64a4394211170e4cefdc5cb159f51a8",
            "f24e007f77578469acf86039a3b329de4bd9e0dd38d575bf46ba44c398948c8c",
            "aa3b906c39b0ec23582769fcd9b886a4bb40c2a063d1dc34a5d6c7ce33a4396a",
            "15c02d40b5a83eceab9377f2f0042b763ceab33d09a6425815ed1acb8ae8a64e",
            "57ee2190466ed9e5ee4b746d881b4265ed00c55eb22361bdfd985b871e95b8d7",
            "6e2c76c8ed7c907c7c314cc29ad29a20e68d694dc59e992b9805dc0bae673a0b",
            "d0e070f854bee56f74d4fff158f4fdefbb6b827b577b612c5a692f74a48721b1",
            "2d6c3a5cf557e2cc4aec5c750d50fd676fad9c0254ee1a4bcfe7cb0cfe3674f1",
            "f07e87c6c30f0cea65e34708440e1bb6f172e3814fcfb3679985a992baaf5c16",
            "b4a6b3d9be53776efc0bdb7b61e8f31b5872cfb55eb1ba488e47c29d5a3c930b",
            "e15ddb6056cc6526b6294feae21674b96713074580bd164623cb56b3df964e64",
            "3fcfbc23de1b7a8db1f7bcd9401128bc0a7a5ff9c712be7dd0bbb657abe4bb82",
            "7fab80feb45f42236ec115f9b380d7826b9b34c30e1b81accdbf93faf254bffa",
            "68e5d996c8c2041f636c4aee66a644218c599c63923d9ed7125db2f0b83533d5",
            "3b19f7f51ea6322ffd819e25264b8c3732e1f61a4c2c6450c59f5e52192a2798",
            "95f8a0b39eb3ed86e58e7b647a65aeb242957fb19d6b587a98fbb040ebfd6c0e",
            "244f769c0465eeb00d82228207573235cb791ae178d7516cc4a1ace7c303cb46",
            "215d33670d6971f0c75f115213ba5509f4056c870d845015b4f34c0287cb775f",
            "9beabd05dd5f8f227d812951fb49079795a05ab081c68e366d27dd41f72e6a7c",
            "da032756b18192594bb037b3c2b2394d6daff9b21ec853ef0922b4b84461f474",
            "59dc5ae5a98e18bc9ad8b61cbed25279752f7e56eb8b4b91bff67fd1f7735229",
            "9c03009f673ec9cca125c47774b2b29e300c6ce4c92805472ba1632b0096a6d6",
            "f79948f1146ea0abcc7acce2220627e44f23df274d639c5d82c32a0c7565b149",
            "1961f55b922950a89da7e3430a89176637129a5b2d5a3bdd2365c691c895558f",
            "20b2eabb508b8f6658f65d84b0e8201a22329daed54aaf2372009592951de169",
            "59e11b6562e6ee95781df5d05fef7a21a25909270d2dec861ec701ffbb6cb1b8",
            "6de296ceeaec67291df85e080d38e46350f079cfaad543bd24865a6c19c29574",
            "92691729968a8b12484d2859f9c9c43f744f83678b8a04422da194267124c140",
            "ffb1265b2a8f596e40d645c9de20faa77ec013481fce59b7cb44a2690fde4246",
            "dbf622f22036d834d3f30385fa2bd1614686256f0bf4ef2609ad100328849d55",
            "0e94afe7e6b09c6a85d318b01e4fcd868cc438f9d70b15b3551037ece8ba6dbd",
            "abb248eb4bdbb4e32771d9f9ca3ac9b32b6c4a0fec0cc1d28460e382cf935d14",
            "6ab5f998c14309e1abb902b229a8c7d5bc24d71a58d1bbd1b95998b48883cd31",
            "01476860e34a5aa8224274a0785bb79c99d3ffe25bd3867ffa616e316d393f91",
            "8fce4d5d97242900844db2b51347790527224a1574f624462578620ab58f84e5",
            "64d53393688e505a33f96af635a11c82ba7b6fafea8755e230a0864ce96fa34a",
            "bc41a658008c891404767b5a676342c8dec4e1d7681cd562f342f4178a881a04",
            "c0dda2b09435ba2c91deaf28a676a9ed382f63d44942a225c910a646259dcb64",
            "7cd8a3428f6e075dac80c7d8e68d365b27145f46e5cc986d8607c154937ed327",
            "132a5be87a955518c5aaba96d74f33363efdcd757a506037eb587d4262282702",
            "0b9a03d4131c521cc956e7d9e7cb0ff7f1d5492093aea54b933739984e59faee",
            "85cee1331c67e540aeb462f5b4be738ba884e82486b8cac2788c37979c93a01b",
            "f0cae4da94c06878efca55dc15297ef7f5d54f7c9d3ede2f74ffa4213db7efc4",
            "f00e2616544a2452d4b5c471f6d7b81f281b73cbe8347d0902f1797e898036d7",
            "4bd9a986c972266df69551d4fe4ca2e07dfbd4fd05d537b37060ad7c1d2559fe",
            "a679eee503461b50a78bd14676156a940abacd8067a73cbc8896bba05b1af404",
            "e96d1ade8e8505ddbef1ded49c56f8481239051347e6c0cf6209416f3264b77a",
            "2f0ea20bf141902b00dfd8a63cb064fc4e3a3940c956020d5f34a09bd99b9188",
            "9f8b65a1331f8e230f3b1758f3abf59d6e3230ea042a287b9e6ae031f892eceb",
            "9af96129080e89bf984cba1865e3010b962f12828adff94d84d1e9ed92f54f76",
            "46ae76adee37bcae590e3a96619d23840329da737236947191e7cefa658cbe2e",
            "a74c5452c08410018ac05603b076fb389f05f75c8cfc57b33ab656e6fc180071",
            "ca0a62b9358ec6a65488bd3871981f55150332e06dfb432d777970391a77049b",
            "d678b86cc31fd7aa0401fb680366194e2368890b25b87f444b0e57172ec727df"
        ]
    }
}
```

</details>


**Example Output**

```
[[356, 96455], [420, 97573], [484, 98655], [548, 99725], [612, 100819], [676, 101877], [740, 102983], [804, 104066], [868, 105136], [932, 106231], [996, 107289], [1060, 108396], [1124, 109467], [1188, 110562], [1252, 111645], [1316, 112728], [1380, 113787], [1444, 114871], [1508, 115978], [1572, 117062], [1636, 118133], [1700, 119217], [1764, 120301], [1828, 121397], [1892, 122469], [1956, 123565], [2020, 124649], [2084, 125733], [2148, 126806], [2212, 127890], [2276, 128987], [2340, 130072], [2404, 131132], [2468, 132205], [2532, 133302], [2596, 134399], [2660, 135472], [2724, 136546], [2788, 137667], [2852, 138753], [2916, 139790], [2980, 140900], [3044, 141998], [3108, 143060], [3172, 144158], [3236, 145244], [3300, 146354], [3364, 147428], [3428, 148526], [3492, 149565], [3556, 150663], [3620, 151774], [3684, 152849], [3748, 153960], [3812, 155047], [3876, 156074], [3940, 157185], [4004, 158296], [4068, 159335], [4132, 160459], [4196, 161558], [4260, 162634], [4324, 163710], [4388, 164834], [4452, 165861], [4516, 166973], [4580, 168098], [4644, 169174], [4708, 170226], [4772, 171315], [4836, 172415], [4900, 173516], [4964, 174604], [5028, 175705], [5092, 176770], [5156, 177871], [5220, 178960], [5284, 180025], [5348, 181139], [5412, 182216], [5476, 183317], [5540, 184335], [5604, 185485], [5668, 186599], [5732, 187676], [5796, 188730], [5860, 189808], [5924, 190935], [5988, 192049], [6052, 193079], [6116, 194218], [6180, 195296], [6244, 196423], [6308, 197430], [6372, 198581], [6436, 199672], [6500, 200703], [6564, 201854], [6628, 202933], [6692, 204024]]
```

---

## Analysis

### Monte-Carlo cross-validation

A single train/validation split can be misleading (especially with clustered `x`). Monte-Carlo cross-validation repeatedly draws random train/validation partitions and averages the resulting errors. It reduces variance from any one lucky (or unlucky) split and gives a distribution of errors per model.

<details> <summary><code>Python3</code> — click to expand</summary>

```python
import json
import time
from typing import Dict, List, Tuple

import numpy as np
import pandas as pd
import matplotlib.pyplot as plt

# --------------------
# Config
# --------------------
CONFIG_PATH = "./gas.json"  # expects {"sepolia": [[x,y], ...], "hoodi": ...}
DATASET = "sepolia"  # switch to "foundry", "hoodi" etc.

# --------------------
# Load (calldata_len, base_gas) pairs
# --------------------
with open(CONFIG_PATH, "r") as f:
    CONFIG = json.load(f)

xy: List[Tuple[int, int]] = CONFIG[DATASET]
x_all = np.array([xx for xx, _ in xy], dtype=float)
y_all = np.array([yy for _, yy in xy], dtype=float)
N = len(x_all)
assert N >= 8, "Need at least 8 points to compare models robustly."
print(f"[INFO] Loaded dataset='{DATASET}' with {N} points from {CONFIG_PATH}")

# --------------------
# Helpers
# --------------------


def fit_predict_poly(x_tr, y_tr, x_va, deg: int) -> np.ndarray:
    """Fit polynomial of given degree, then predict on validation x."""
    coeffs = np.polyfit(x_tr, y_tr, deg)
    p = np.poly1d(coeffs)
    return p(x_va)


def metrics(y_true, y_pred) -> Dict[str, float]:
    err = y_true - y_pred
    abs_err = np.abs(err)
    return {
        "mae": float(np.mean(abs_err)),
        "rmse": float(np.sqrt(np.mean(err**2))),
        "min_err": float(np.min(abs_err)),
        "max_err": float(np.max(abs_err)),
    }


def monte_carlo_cv(
    x: np.ndarray,
    y: np.ndarray,
    degrees=(1, 2, 3),
    train_ratio: float = 0.7,
    trials: int = 1000,
    seed: int = 12345,
) -> Dict[int, Dict[str, float]]:
    """
    Repeated random subsampling CV:
      - Randomly split (train/val) 'trials' times
      - Fit each degree on train, evaluate on validation
      - Aggregate mean/std metrics and win counts (rmse/mae/max)
    """
    rng = np.random.default_rng(seed)
    N = len(x)
    results = {
        d: {
            "rmse": [],
            "mae": [],
            "min_err": [],
            "max_err": [],
            "wins": {"rmse": 0, "mae": 0, "max_err": 0},
        }
        for d in degrees
    }

    train_size = int(round(train_ratio * N))
    if train_size < 2:
        raise ValueError("Train set too small.")
    for d in degrees:
        if train_size < d + 1:
            raise ValueError(f"Train size {train_size} too small for degree {d}")

    for _ in range(trials):
        idx = rng.permutation(N)
        tr_idx = idx[:train_size]
        va_idx = idx[train_size:]
        if len(va_idx) == 0:
            continue

        x_tr, y_tr = x[tr_idx], y[tr_idx]
        x_va, y_va = x[va_idx], y[va_idx]

        trial = {}
        for d in degrees:
            y_hat = fit_predict_poly(x_tr, y_tr, x_va, d)
            m = metrics(y_va, y_hat)
            for k in ("rmse", "mae", "min_err", "max_err"):
                results[d][k].append(m[k])
            trial[d] = m

        # Count winners per metric (lower is better)
        rmse_winner = min(degrees, key=lambda d: trial[d]["rmse"])
        mae_winner = min(degrees, key=lambda d: trial[d]["mae"])
        max_winner = min(
            degrees, key=lambda d: trial[d]["max_err"]
        )  # minimize worst-case
        results[rmse_winner]["wins"]["rmse"] += 1
        results[mae_winner]["wins"]["mae"] += 1
        results[max_winner]["wins"]["max_err"] += 1

    # Aggregate
    out: Dict[int, Dict[str, float]] = {}
    for d in degrees:
        r = results[d]
        rmse_arr = np.array(r["rmse"], dtype=float)
        mae_arr = np.array(r["mae"], dtype=float)
        min_arr = np.array(r["min_err"], dtype=float)
        max_arr = np.array(r["max_err"], dtype=float)
        n_eff = max(1, len(rmse_arr))

        out[d] = {
            "mean_rmse": float(np.mean(rmse_arr)) if n_eff else float("inf"),
            "std_rmse": float(np.std(rmse_arr, ddof=1)) if n_eff > 1 else 0.0,
            "mean_mae": float(np.mean(mae_arr)) if n_eff else float("inf"),
            "std_mae": float(np.std(mae_arr, ddof=1)) if n_eff > 1 else 0.0,
            "mean_min": float(np.mean(min_arr)) if n_eff else float("inf"),
            "mean_max": float(np.mean(max_arr)) if n_eff else float("inf"),
            "win_rmse_pct": 100.0 * r["wins"]["rmse"] / trials,
            "win_mae_pct": 100.0 * r["wins"]["mae"] / trials,
            "win_max_pct": 100.0 * r["wins"]["max_err"] / trials,
        }
    return out


def sweep_train_ratios(
    x: np.ndarray,
    y: np.ndarray,
    degrees=(1, 2, 3),
    ratios=np.round(np.arange(0.10, 0.91, 0.01), 2),
    trials=800,
    seed=None,
    dataset_name: str = "unknown",
) -> pd.DataFrame:
    """
    For each train_ratio in ratios, run Monte Carlo CV and record summary stats.
    Returns long-form DataFrame with columns including dataset_name.
    Prints simple progress for long runs.
    """
    if seed is None:
        seed = int(time.time())
    rows = []
    total = len(ratios)
    start = time.time()
    for i, r in enumerate(ratios, start=1):
        summary = monte_carlo_cv(
            x,
            y,
            degrees=degrees,
            train_ratio=float(r),
            trials=trials,
            seed=seed + int(r * 100),
        )
        for d in degrees:
            s = summary[d]
            rows.append(
                {
                    "dataset": dataset_name,
                    "train_ratio": float(r),
                    "degree": d,
                    "mean_rmse": s["mean_rmse"],
                    "std_rmse": s["std_rmse"],
                    "mean_mae": s["mean_mae"],
                    "std_mae": s["std_mae"],
                    "win_rmse_pct": s["win_rmse_pct"],
                    "win_mae_pct": s["win_mae_pct"],
                    "win_max_pct": s["win_max_pct"],
                }
            )
        # Progress log (concise)
        elapsed = time.time() - start
        print(
            f"[{i}/{total}] dataset={dataset_name} ratio={r:.2f} | elapsed={elapsed:.1f}s"
        )

    return pd.DataFrame(rows)


# --------------------
# Run the sweep with progress
# --------------------
ratios = np.round(np.arange(0.10, 0.91, 0.01), 2)
df = sweep_train_ratios(
    x_all,
    y_all,
    degrees=(1, 2, 3),
    ratios=ratios,
    trials=20_000,  # adjust for speed vs variance
    seed=int(time.time()),
    dataset_name=DATASET,
)

# Winners per train_ratio (lowest mean_rmse)
winners = (
    df.loc[
        df.groupby(["dataset", "train_ratio"])["mean_rmse"].idxmin(),
        ["dataset", "train_ratio", "degree", "mean_rmse"],
    ]
    .sort_values(["dataset", "train_ratio"])
    .reset_index(drop=True)
)

# Save results (tagged with dataset)
df.to_csv(f"{DATASET}_mc_cv_sweep.csv", index=False)
winners.to_csv(f"{DATASET}_mc_cv_winners.csv", index=False)

# --------------------
# Visualizations
# --------------------

# 1) Mean RMSE vs train_ratio for each degree (single chart)
plt.figure()
for d in sorted(df["degree"].unique()):
    sub = df[(df["degree"] == d) & (df["dataset"] == DATASET)].sort_values(
        "train_ratio"
    )
    plt.plot(sub["train_ratio"], sub["mean_rmse"], label=f"deg={d}")
plt.xlabel("Train ratio")
plt.ylabel("Mean RMSE (gas)")
plt.title(f"{DATASET}: Mean RMSE vs Train Ratio by Degree")
plt.legend()
plt.tight_layout()
plt.show()

# 2) “Candlestick” style: error bars with mean ± 1 std (one chart per degree)
for d in sorted(df["degree"].unique()):
    sub = df[(df["degree"] == d) & (df["dataset"] == DATASET)].sort_values(
        "train_ratio"
    )
    plt.figure()
    plt.errorbar(
        sub["train_ratio"], sub["mean_rmse"], yerr=sub["std_rmse"], fmt="o", capsize=2
    )
    plt.xlabel("Train ratio")
    plt.ylabel("RMSE (gas)")
    plt.title(f"{DATASET}: Degree {d} RMSE mean ± 1 std")
    plt.tight_layout()
    plt.show()

# 3) Combined "candlestick" view: mean ± 1 std for all degrees on one chart
plt.figure()
markers = ['o', 's', '^']  # will cycle if you add more degrees
for idx, d in enumerate(sorted(df["degree"].unique())):
    sub = df[(df["degree"] == d) & (df["dataset"] == DATASET)].sort_values("train_ratio")
    plt.errorbar(
        sub["train_ratio"],
        sub["mean_rmse"],
        yerr=sub["std_rmse"],
        fmt=markers[idx % len(markers)],
        capsize=2,
        label=f"deg={d}",
    )
plt.xlabel("Train ratio")
plt.ylabel("RMSE (gas)")
plt.title(f"{DATASET}: RMSE mean ± 1 std (all degrees)")
plt.legend()
plt.tight_layout()
plt.show()


# 4) (Optional) also print who “wins” at each ratio
print(f"\n{'train_ratio':>11} | {'winner_deg':>10} | {'mean_rmse':>12}")
print("-" * 40)
for _, row in winners.iterrows():
    print(
        f"{row['train_ratio']:>11.2f} | {int(row['degree']):>10d} | {row['mean_rmse']:>12.2f}"
    )
```

</details>


<details> <summary><code>gas.json</code> — click to expand</summary>

```json
{
    "sepolia": [
        [356, 96455],
        [420, 97561],
        [484, 98655],
        [548, 99737],
        [612, 100807],
        [676, 101901],
        [740, 102983],
        [804, 104066],
        [868, 105148],
        [932, 106231],
        [996, 107313],
        [1060, 108396],
        [1124, 109479],
        [1188, 110538],
        [1252, 111645],
        [1316, 112716],
        [1380, 113811],
        [1444, 114847],
        [1508, 115978],
        [1572, 117038],
        [1636, 118145],
        [1700, 119229],
        [1764, 120277],
        [1828, 121385],
        [1892, 122457],
        [1956, 123553],
        [2020, 124625],
        [2084, 125721],
        [2148, 126806],
        [2212, 127890],
        [2276, 128963],
        [2340, 130060],
        [2404, 131156],
        [2468, 132241],
        [2532, 133326],
        [2596, 134411],
        [2660, 135496],
        [2724, 136558],
        [2788, 137667],
        [2852, 138741],
        [2916, 139838],
        [2980, 140924],
        [3044, 141974],
        [3108, 143084],
        [3172, 144182],
        [3236, 145244],
        [3300, 146354],
        [3364, 147440],
        [3428, 148526],
        [3492, 149565],
        [3556, 150687],
        [3620, 151774],
        [3684, 152849],
        [3748, 153888],
        [3812, 155035],
        [3876, 156110],
        [3940, 157209],
        [4004, 158284],
        [4068, 159371],
        [4132, 160459],
        [4196, 161534],
        [4260, 162646],
        [4324, 163734],
        [4388, 164810],
        [4452, 165873],
        [4516, 166949],
        [4580, 168074],
        [4644, 169150],
        [4708, 170250],
        [4772, 171339],
        [4836, 172439],
        [4900, 173492],
        [4964, 174616],
        [5028, 175693],
        [5092, 176806],
        [5156, 177895],
        [5220, 178960],
        [5284, 180049],
        [5348, 181127],
        [5412, 182240],
        [5476, 183305],
        [5540, 184383],
        [5604, 185485],
        [5668, 186575],
        [5732, 187616],
        [5796, 188766],
        [5860, 189832],
        [5924, 190923],
        [5988, 192037],
        [6052, 193127],
        [6116, 194158],
        [6180, 195248],
        [6244, 196351],
        [6308, 197478],
        [6372, 198593],
        [6436, 199648],
        [6500, 200775],
        [6564, 201854],
        [6628, 202945],
        [6692, 204024]
    ],
    "hoodi": [
        [356, 96455],
        [420, 97573],
        [484, 98655],
        [548, 99725],
        [612, 100819],
        [676, 101877],
        [740, 102983],
        [804, 104066],
        [868, 105136],
        [932, 106231],
        [996, 107289],
        [1060, 108396],
        [1124, 109467],
        [1188, 110562],
        [1252, 111645],
        [1316, 112728],
        [1380, 113787],
        [1444, 114871],
        [1508, 115978],
        [1572, 117062],
        [1636, 118133],
        [1700, 119217],
        [1764, 120301],
        [1828, 121397],
        [1892, 122469],
        [1956, 123565],
        [2020, 124649],
        [2084, 125733],
        [2148, 126806],
        [2212, 127890],
        [2276, 128987],
        [2340, 130072],
        [2404, 131132],
        [2468, 132205],
        [2532, 133302],
        [2596, 134399],
        [2660, 135472],
        [2724, 136546],
        [2788, 137667],
        [2852, 138753],
        [2916, 139790],
        [2980, 140900],
        [3044, 141998],
        [3108, 143060],
        [3172, 144158],
        [3236, 145244],
        [3300, 146354],
        [3364, 147428],
        [3428, 148526],
        [3492, 149565],
        [3556, 150663],
        [3620, 151774],
        [3684, 152849],
        [3748, 153960],
        [3812, 155047],
        [3876, 156074],
        [3940, 157185],
        [4004, 158296],
        [4068, 159335],
        [4132, 160459],
        [4196, 161558],
        [4260, 162634],
        [4324, 163710],
        [4388, 164834],
        [4452, 165861],
        [4516, 166973],
        [4580, 168098],
        [4644, 169174],
        [4708, 170226],
        [4772, 171315],
        [4836, 172415],
        [4900, 173516],
        [4964, 174604],
        [5028, 175705],
        [5092, 176770],
        [5156, 177871],
        [5220, 178960],
        [5284, 180025],
        [5348, 181139],
        [5412, 182216],
        [5476, 183317],
        [5540, 184335],
        [5604, 185485],
        [5668, 186599],
        [5732, 187676],
        [5796, 188730],
        [5860, 189808],
        [5924, 190935],
        [5988, 192049],
        [6052, 193079],
        [6116, 194218],
        [6180, 195296],
        [6244, 196423],
        [6308, 197430],
        [6372, 198581],
        [6436, 199672],
        [6500, 200703],
        [6564, 201854],
        [6628, 202933],
        [6692, 204024]
    ],
    "foundry": [
        [356, 113559],
        [420, 114298],
        [484, 115065],
        [548, 115835],
        [612, 116605],
        [676, 117375],
        [740, 118145],
        [804, 118916],
        [868, 119686],
        [932, 120457],
        [996, 121227],
        [1060, 121998],
        [1124, 122769],
        [1188, 123540],
        [1252, 124311],
        [1316, 125082],
        [1380, 125853],
        [1444, 126625],
        [1508, 127396],
        [1572, 128168],
        [1636, 128939],
        [1700, 129711],
        [1764, 130483],
        [1828, 131255],
        [1892, 132027],
        [1956, 132799],
        [2020, 133571],
        [2084, 134343],
        [2148, 135116],
        [2212, 135888],
        [2276, 136661],
        [2340, 137434],
        [2404, 138206],
        [2468, 138979],
        [2532, 139752],
        [2596, 140525],
        [2660, 141298],
        [2724, 142072],
        [2788, 142845],
        [2852, 143619],
        [2916, 144392],
        [2980, 145166],
        [3044, 145940],
        [3108, 146714],
        [3172, 147488],
        [3236, 148262],
        [3300, 149036],
        [3364, 149810],
        [3428, 150584],
        [3492, 151359],
        [3556, 152133],
        [3620, 152908],
        [3684, 153683],
        [3748, 154458],
        [3812, 155233],
        [3876, 156008],
        [3940, 156783],
        [4004, 157558],
        [4068, 158333],
        [4132, 159109],
        [4196, 159884],
        [4260, 160660],
        [4324, 161436],
        [4388, 162212],
        [4452, 162987],
        [4516, 163763],
        [4580, 164540],
        [4644, 165316],
        [4708, 166092],
        [4772, 166869],
        [4836, 167645],
        [4900, 168422],
        [4964, 169198],
        [5028, 169975],
        [5092, 170752],
        [5156, 171529],
        [5220, 172306],
        [5284, 173083],
        [5348, 173861],
        [5412, 174638],
        [5476, 175415],
        [5540, 176193],
        [5604, 176971],
        [5668, 177749],
        [5732, 178526],
        [5796, 179304],
        [5860, 180082],
        [5924, 180861],
        [5988, 181639],
        [6052, 182417],
        [6116, 183196],
        [6180, 183974],
        [6244, 184753],
        [6308, 185532],
        [6372, 186311],
        [6436, 187090],
        [6500, 187869],
        [6564, 188648],
        [6628, 189427],
        [6692, 190206],
        [6756, 190986],
        [6820, 191765],
        [6884, 192545],
        [6948, 193325],
        [7012, 194104],
        [7076, 194884],
        [7140, 195664],
        [7204, 196445],
        [7268, 197225],
        [7332, 198005],
        [7396, 198785],
        [7460, 199566],
        [7524, 200347],
        [7588, 201127],
        [7652, 201908],
        [7716, 202689],
        [7780, 203470],
        [7844, 204251],
        [7908, 205032],
        [7972, 205814],
        [8036, 206595],
        [8100, 207377],
        [8164, 208158],
        [8228, 208940],
        [8292, 209722],
        [8356, 210503],
        [8420, 211285],
        [8484, 212068],
        [8548, 212850],
        [8612, 213632],
        [8676, 214414],
        [8740, 215197],
        [8804, 215979],
        [8868, 216762],
        [8932, 217545],
        [8996, 218328],
        [9060, 219111],
        [9124, 219894],
        [9188, 220677],
        [9252, 221460],
        [9316, 222243],
        [9380, 223027],
        [9444, 223810],
        [9508, 224594],
        [9572, 225378],
        [9636, 226162],
        [9700, 226946],
        [9764, 227730],
        [9828, 228514],
        [9892, 229298],
        [9956, 230082],
        [10020, 230867],
        [10084, 231651],
        [10148, 232436],
        [10212, 233220],
        [10276, 234005],
        [10340, 234790],
        [10404, 235575],
        [10468, 236360],
        [10532, 237146],
        [10596, 237931],
        [10660, 238716],
        [10724, 239502],
        [10788, 240287],
        [10852, 241073],
        [10916, 241859],
        [10980, 242645],
        [11044, 243431],
        [11108, 244217],
        [11172, 245003],
        [11236, 245789],
        [11300, 246576],
        [11364, 247362],
        [11428, 248149],
        [11492, 248935],
        [11556, 249722],
        [11620, 250509],
        [11684, 251296],
        [11748, 252083],
        [11812, 252870],
        [11876, 253657],
        [11940, 254445],
        [12004, 255232],
        [12068, 256020],
        [12132, 256807],
        [12196, 257595],
        [12260, 258383],
        [12324, 259171],
        [12388, 259959],
        [12452, 260747],
        [12516, 261535],
        [12580, 262324],
        [12644, 263112],
        [12708, 263900],
        [12772, 264689],
        [12836, 265478],
        [12900, 266267],
        [12964, 267056],
        [13028, 267845],
        [13092, 268634]
    ]
}
```

</details>

### What we vary

* **Model family**: degree-1 (linear), degree-2 (quadratic), degree-3 (cubic) polynomials of `x`.
* **Train size**: we sweep `train_ratio` from **0.10 → 0.90** (step 0.01). This shows how each model behaves when you have only a few points versus many.
* **Random splits**: for each ratio we run **many trials** (e.g., 20k) with different random partitions to sample the error distribution.

Constraint: a degree-`d` polynomial needs at least `d+1` training points; the code enforces this.

### What we measure

For each split and each model we compute:

* **RMSE (Root Mean Squared Error)** — sensitive to larger mistakes, good proxy for *typical squared loss*.
* **MAE (Mean Absolute Error)** — robust to outliers; measures *typical absolute miss* in gas units.
* **Min/Max absolute error** — best/worst cases across the validation slice (guardrails).

Across thousands of splits we then report **mean ± std** for RMSE/MAE and **win rates** (fraction of splits where a model is best). We try to find:

* Which model has **lower average error**?
* How **stable** is that performance (standard deviation)?
* How often does each model **win** outright?

### Interpreting the figures

1. **Mean RMSE vs Train Ratio (per degree, one chart)**
   Shows how each model’s *typical* error falls as you feed it more data. Expect curves to drop then flatten.

   * If **linear** stays above **quadratic** for most ratios, linear likely **underfits**.
   * If **cubic** only wins at very high ratios and has larger variance, it likely **overfits** for small/mid data.

2. **Candlesticks (mean ± 1 std) per degree**
   Each degree gets its own “error-bar” chart across train ratios. Smaller bars = more **stable** generalization.

3. **Combined candlesticks (all degrees)**
   Puts all degrees on one plot so you can compare error levels **and** error bars at a glance for each ratio.

4. **Winners table**
   For each train ratio, we also print which degree has the lowest **mean RMSE**. This is a quick summary of who’s “best” as data availability changes.

### Model-selection

We pick the degree with the **lowest mean RMSE** at the relevant train ratio.
**Tie-break:** if a *simpler* model’s mean RMSE is within **1 standard deviation** of the best, we choose the simpler model.

### Why quadratic tends to win

* **Linear term** dominates due to calldata pricing (≈16 gas per non-zero byte, ≈4 per zero byte).
* **Quadratic term** is typically **small but real** from the Yellow Paper’s memory cost (roughly linear + quadratic in memory words). As payload grows, decoding/copying arrays can trigger modest memory expansion.
* **Cubic** rarely has a consistent physical justification here; when it “wins,” it often reflects overfitting unless you have a *lot* of spread and strong cubic-like effects (uncommon for our case).

### Notices

* **Trials:** more trials → tighter estimates (but longer runtime). We use thousands to average out split noise.
* **Multiple datasets:** the analysis tags results with the dataset name (e.g., `foundry`, `sepolia`, `hoodi`) to compare environments.

### Apply the choosen model

1. Persist the winning coefficients (usually **quadratic**) for the target dataset.
2. Pack them on-chain in your chosen fixed-width layout (e.g., 3×80-bit lanes).
3. Use

   ```
   baseGas(x) ≈ c2*x^2 + c1*x + c0
   TotalTxGasUsed ≈ baseGas(x) + targetGasUsed
   ```

   so the relayer can front minimal native token and be **made whole** from the consumer vault.
4. Monitor drift: if calldata patterns or EVM changes shift the curve, perform recalibration to update the coefficients.

---

## Results

This section summarizes the Monte-Carlo cross-validation over the dataset of `(calldata_size, baseGas)` pairs, where `baseGas(x) = tx.gasUsed − targetGasUsed`. We sweep the training ratio from **0.10 → 0.90** (step **0.01**), and for each split fit **linear (deg=1)**, **quadratic (deg=2)**, and **cubic (deg=3)** models of `baseGas(x)`.

### Dataset & setup

* **Dataset:** `sepolia` (example shown below; multiple datasets can be run independently and compared)
* **Train/val splits:** Monte-Carlo random splits per train ratio
* **Trials per ratio:** large (e.g., 5,000) to stabilize mean/std
* **Metrics per model & ratio:** **Mean RMSE**, **Std RMSE**, **Mean MAE**, **Std MAE**, plus **win rates** (how often a degree wins for RMSE/MAE/MaxErr across trials)

---

### Sweep results

> Each row aggregates all trials for the given `(dataset, train_ratio, degree)`.

| dataset | train\_ratio | degree |    mean\_rmse |     std\_rmse |     mean\_mae |      std\_mae | win\_rmse\% | win\_mae\% | win\_max\% |
| :------ | -----------: | -----: | ------------: | ------------: | ------------: | ------------: | -------------: | ------------: | ------------: |
| sepolia |         0.10 |      1 | 44.7386417866 |  7.6343655385 | 35.1602731464 |  4.7381375849 |           0.13 |          0.13 |         0.205 |
| sepolia |         0.10 |      2 | 21.2250749941 |  5.7972672391 | 16.0636191321 |  4.3019593741 |          75.97 |        75.715 |        68.655 |
| sepolia |         0.10 |      3 | 27.8902636709 | 30.8944809159 | 20.0280887839 | 18.0000260342 |          23.90 |        24.155 |        31.140 |
| sepolia |         0.11 |      1 | 44.0434613733 |  6.7225464004 | 34.7236781733 |  4.1579090745 |          0.055 |         0.050 |         0.080 |
| sepolia |         0.11 |      2 | 20.6502731990 |  4.1331709345 | 15.6432875606 |  3.2047751314 |         74.885 |        74.970 |        67.285 |
| sepolia |         0.11 |      3 | 25.5023045863 | 21.2571951118 | 18.5821654316 | 12.4587067921 |         25.060 |        24.980 |        32.635 |
| sepolia |         0.12 |      1 | 43.5571788577 |  6.1565873814 | 34.4099688681 |  3.8289671593 |          0.020 |         0.025 |         0.045 |
| sepolia |         0.12 |      2 | 20.2754642191 |  3.5078912663 | 15.3429774572 |  2.7579781805 |         74.585 |        74.810 |        66.145 |
| sepolia |         0.12 |      3 | 23.9242877511 | 14.4376618507 | 17.5990728822 |  8.5492449054 |         25.395 |        25.165 |        33.810 |
| sepolia |         0.13 |      1 | 43.1188564250 |  5.7363634942 | 34.1460924103 |  3.5322053243 |          0.000 |         0.000 |         0.005 |
| sepolia |         0.13 |      2 | 19.9605986747 |  2.9253350885 | 15.1097822145 |  2.3397913744 |         73.425 |        73.970 |        64.835 |
| sepolia |         0.13 |      3 | 22.9049740714 | 11.5312753511 | 16.9473485322 |  6.8629437007 |         26.575 |        26.030 |        35.160 |
| …       |            … |      … |             … |             … |             … |             … |              … |             … |             … |

> **Observation:** Quadratic (deg=2) consistently exhibits the **lowest mean RMSE/MAE** with **moderate variance**, while cubic (deg=3) has **larger std** (instability/overfitting) and linear (deg=1) **underfits**.

---

### Per-ratio winners (RMSE)

For each `train_ratio`, the degree with the lowest **mean RMSE**:

| dataset | train\_ratio | degree |    mean\_rmse |
| :------ | -----------: | -----: | ------------: |
| sepolia |         0.10 |      2 | 21.2250749941 |
| sepolia |         0.11 |      2 | 20.6502731990 |
| sepolia |         0.12 |      2 | 20.2754642191 |
| sepolia |         0.13 |      2 | 19.9605986747 |
| sepolia |         0.14 |      2 | 19.7072057299 |
| sepolia |         0.15 |      2 | 19.5314136854 |
| sepolia |         0.16 |      2 | 19.3401813605 |
| sepolia |         0.17 |      2 | 19.2209853393 |
| sepolia |         0.18 |      2 | 19.0879224237 |
| sepolia |         0.19 |      2 | 19.0018219507 |
| sepolia |         0.20 |      2 | 18.8786459713 |
| sepolia |         0.21 |      2 | 18.7947708730 |
| sepolia |         0.22 |      2 | 18.6942663442 |
| …       |            … |      … |             … |

> **Takeaway:** Across the sweep, **quadratic** is the consistent winner for RMSE on this dataset.

---

### Figures

The following images are produced by the analysis script and should be embedded in the repo.

1. **Mean RMSE vs Train Ratio (deg=1,2,3)**
   ![RMSE vs Train Ratio](./images/rmse_vs_ratio.png)
   *Quadratic (green) sits below linear and cubic across most ratios.*
   * **Blue (deg=1 / linear)**: always the worst; high bias. Starts \~**45 gas** RMSE at 10% train, drifts down to \~**38–39 gas** at 90%. It ignores the **EVM memory expansion**, so its error stays high even with lots of data.
   * **Orange (deg=2 / quadratic)**: consistently accurate across the whole sweep. Starts \~**21 gas**, slides smoothly to \~**17.2–17.4 gas** at 90%.
   * **Green (deg=3 / cubic)**: unstable with little data (≈**26–28 gas** at 10%), improves quickly and **approaches** quadratic for large train ratios. With small training sets it overfits (big variance → high RMSE). As the training set grows, it stabilizes and gets close to quadratic, but it **doesn’t deliver a systematic gain**

2. **Candlestick (mean ± std RMSE) — Linear**
   ![Linear Candlestick](./images/linear_candlestick.png)
   *Linear shows higher mean and relatively middle variance (consistent underfit).*
   * The mean RMSE starts ≈ **45 gas** at `train_ratio ≈ 0.10` and only drifts down to ≈ **38–39 gas** by `0.90`.
   * The error bars (±1 std) are wide (high variance) at the small train ratios and large train ratios (≈0.10 and ≈0.90), narrowest around **0.45–0.60**.

3. **Candlestick (mean ± std RMSE) — Quadratic**
   ![Quadratic Candlestick](./images/quadratic_candlestick.png)
   *Quadratic exhibits the best bias–variance trade-off: low mean, moderate std.*
   * The mean RMSE starts around **≈21 gas** at `train_ratio ≈ 0.10` and steadily falls to **≈17 gas** by `0.90`.
   * The largest improvement is between **0.10 → ~0.30** (roughly **2–3 gas** reduction). Beyond **\~0.40**, improvements are slowing down.
   * Lowest variance in the middle (**~0.30 → ~0.40**). The error bars (±1 std) are widest near 0.1 and 0.9 and **narrowest around \~0.35**:

4. **Candlestick (mean ± std RMSE) — Cubic**
   ![Cubic Candlestick](./images/cubic_candlestick.png)
   *Cubic’s mean can be close at some ratios but with noticeably larger std (overfit risk).*
   * Around `train_ratio ≈ 0.10–0.15`, the mean RMSE is \~**28–25 gas**, but the **std is large** (upper whiskers > **60 gas**, lower whiskers dip near 0).
   * As the train set adds points across the `x` range, variance collapses and the mean RMSE drops to **\~19–20 gas**.
   * From **0.30 → 0.90**, the mean slowly improves to **\~17–18 gas**, roughly on par with degree-2. Error bars remain modest (≈±1.5–2.5 gas), then widen slightly at the high train ratios.

5. **Combined Candlesticks (deg=1,2,3)**
   ![Combined Candlestick](./images/combined_candlestick.png)
   *All three in one frame for direct comparison; quadratic is lowest and most stable overall.*
   * **Blue (deg=1 / linear)**: Exhibits a consistently highest average error (~40 gas). Although its error bars narrow as the training ratio increases, its systematic bias from underfitting persists.
   * **Orange (deg=2 / quadratic)**: Achieves the lowest mean RMSE across almost all training ratios (from 0.10 to 0.90) and maintains tight error bars.
   * **Green (deg=3 / cubic)**: Displays high variance (wide error bars) at low training ratios. As more data is used for training, its performance converges toward that of the quadratic model. However, its standard deviation remains consistently larger.

### What this means for `baseGas(x)` in production

* Our production model for **`baseGas(x)`** should default to a **quadratic polynomial** of calldata length `x`.
* We pack the learned coefficients on-chain (router) to adjust fees so the relayer can be reimbursed from the consumer’s vault, enabling **low-balance continuous relaying**.
* The candlesticks indicate the **expected variability** across random splits.

---

## Appendix: EVM Cost Anatomy

* **Intrinsic base**: `21000`.

* **Calldata** (post-Istanbul): `16` gas per non-zero byte, `4` per zero byte.

* **Memory expansion**:

  See [Yellow Paper](https://ethereum.github.io/yellowpaper/paper.pdf)

  ```
  C_mem(a) = G_memory * a + floor(a^2 / 512)
  DeltaC   = C_mem(a1) - C_mem(a0)
  ```

  where `a` is memory size in 32-byte words. The `a^2/512` term explains the mild quadratic growth.

* **Cold vs warm** (EIP-2929): the first access to an account/storage slot in a tx is costlier; subsequent accesses are cheaper. This yields intercept shifts and small residuals, not a change to the fundamental shape.

---

## Summary

* We estimate **TotalTxGasUsed** by **adding** the measured **consumer work** (`targetGasUsed`) to a learned **overhead model** `f(x)` that depends on **calldata length**.
* **Quadratic** is typically the best model for `f(x)`, consistent with **memory expansion** and per-entry loops; Linear tends to underfit, Cubic tends to overfit.
* The **Foundry harness** (snapshot/revert, memory reset) yields repeatable intra-repo measurements and supports on-chain **calibration** (e.g., `additionalGasUsed`).
* The **Python Monte-Carlo** sweep quantifies model uncertainty across train ratios and prefers simpler models when statistically indistinguishable.
* With this pipeline, a relayer can operate with *near-zero native balance*, being reimbursed **for the entire tx** from the consumer’s vault, and you can keep the model fresh as conditions evolve.
