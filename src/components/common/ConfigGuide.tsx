import { AlertTriangle, FileText, Key, Settings } from "lucide-react";

interface ConfigGuideProps {
  errorMessage?: string;
}

const CONFIG_EXAMPLE = `{
  "providers": {
    "qwen": {
      "apiKey": "sk-your-dashscope-api-key",
      "models": {
        "qwen-image-plus": {
          "url": "https://dashscope.aliyuncs.com/api/v1/services/aigc/text2image/image-synthesis"
        },
        "qwen-image-edit-max": {
          "url": "https://dashscope.aliyuncs.com/api/v1/services/aigc/multimodal-generation/generation"
        },
        "qwen-mt-image": {
          "url": "https://dashscope.aliyuncs.com/api/v1/services/aigc/image2image/image-synthesis"
        }
      }
    }
  }
}`;

export function ConfigGuide({ errorMessage }: ConfigGuideProps) {
  return (
    <div className="flex flex-1 items-center justify-center p-8">
      <div className="max-w-lg space-y-6">
        {/* Header */}
        <div className="text-center">
          <Settings className="mx-auto h-12 w-12 text-muted-foreground" />
          <h2 className="mt-3 text-xl font-semibold text-foreground">
            Configuration Required
          </h2>
          <p className="mt-1 text-sm text-muted-foreground">
            QwenImager needs a configuration file to connect to the DashScope
            API.
          </p>
        </div>

        {/* Error message */}
        {errorMessage && (
          <div className="flex gap-3 rounded-lg border border-destructive/50 bg-destructive/10 p-4">
            <AlertTriangle className="h-5 w-5 shrink-0 text-destructive" />
            <div className="text-sm text-destructive">{errorMessage}</div>
          </div>
        )}

        {/* Steps */}
        <div className="space-y-4">
          <h3 className="text-sm font-medium text-foreground">Setup Steps:</h3>

          <div className="space-y-3">
            <div className="flex gap-3">
              <div className="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-primary text-xs font-bold text-primary-foreground">
                1
              </div>
              <div>
                <p className="text-sm font-medium text-foreground">
                  Get a DashScope API Key
                </p>
                <p className="text-xs text-muted-foreground">
                  Visit dashscope.aliyuncs.com to create an API key
                </p>
              </div>
            </div>

            <div className="flex gap-3">
              <div className="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-primary text-xs font-bold text-primary-foreground">
                2
              </div>
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-2">
                  <FileText className="h-4 w-4 text-muted-foreground" />
                  <p className="text-sm font-medium text-foreground">
                    Create configuration file
                  </p>
                </div>
                <p className="mt-1 text-xs text-muted-foreground">
                  Create the file at:
                </p>
                <code className="mt-1 block rounded bg-muted px-2 py-1 text-xs text-foreground">
                  ~/.qwenimage/setting.json
                </code>
              </div>
            </div>

            <div className="flex gap-3">
              <div className="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-primary text-xs font-bold text-primary-foreground">
                3
              </div>
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-2">
                  <Key className="h-4 w-4 text-muted-foreground" />
                  <p className="text-sm font-medium text-foreground">
                    Add your API key and models
                  </p>
                </div>
                <p className="mt-1 text-xs text-muted-foreground">
                  Paste the following JSON and replace the API key:
                </p>
                <pre className="mt-2 overflow-x-auto rounded-lg bg-muted p-3 text-xs text-foreground">
                  {CONFIG_EXAMPLE}
                </pre>
              </div>
            </div>

            <div className="flex gap-3">
              <div className="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-primary text-xs font-bold text-primary-foreground">
                4
              </div>
              <div>
                <p className="text-sm font-medium text-foreground">
                  Restart the application
                </p>
                <p className="text-xs text-muted-foreground">
                  Close and reopen QwenImager to load the configuration
                </p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
