# formulafix

A CLI based tool written in Rust. This tool can be used to discard or pull back any workFlow in IPA application.

## Usage

To discard a workflow -

```
./formulafix -C <IPA_DB_URL> --discard --workFlowId <WORKFLOW_ID> --flowTypeId <FLOW_TYPE_ID>
```

To pull back a workflow -

```
./formulafix -C <IPA_DB_URL> --pull-back --stage <STAGE(1 or 3)> --workFlowId <WORKFLOW_ID> --flowTypeId <FLOW_TYPE_ID>
```
