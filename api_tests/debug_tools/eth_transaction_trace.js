const ethutil = require("ethereumjs-util");
const Web3 = require("web3");
const Api = require("@parity/api");

const provider = new Api.Provider.Http('http://localhost:33056');
const api = new Api(provider);

const web3 = new Web3(new Web3.providers.HttpProvider("http://localhost:33056"));

async function printTx (txid) {
    const tx = await web3.eth.getTransaction(txid);
    console.log("Transaction:", tx);
}

async function printReceipt (txid) {
    const receipt = await web3.eth.getTransactionReceipt(txid);
    console.log("Receipt:", receipt);
}

async function traceReplay (txid) {
    return api.trace.replayTransaction(txid).then((traces) => {
        console.log("Trace Transaction Replay:", traces);
    });
}

async function traceTransaction (txid) {
    const trace = await api.trace.transaction(txid);
    console.log("Trace Transaction:", trace);
}

async function traceCall () {
    api.trace.call({
        from: '0x00a329c0648769A73afAc7F9381E08FB43dBEA72',
        gas: 918296,
        data: '0x000000000000000000000000000000000000000000000001020304060607090a',
        to: '0xe46FB33e4DB653De84cB0E0E8b810A6c4cD39d59',
        value: 0
    });
    console.log("Trace Call:", trace);
}

const tx_id = "0xc2983c95eadda736c97569f8d2388ac7d268bde14b4d4abf1ebb95c8ea47270e";

// printTx(tx_id);
// printReceipt(tx_id);
traceReplay(tx_id);
// traceTransaction(tx_id);
// traceCall();
