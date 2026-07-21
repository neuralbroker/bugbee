import path from "path"

process.env.BUGBEE_DB = ":memory:"
process.env.BUGBEE_MODELS_PATH = path.join(import.meta.dir, "plugin", "fixtures", "models-dev.json")
process.env.BUGBEE_DISABLE_MODELS_FETCH = "true"
