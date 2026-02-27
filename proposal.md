# 核心需求

- 利用Tauri开发一款跨平台的桌面App，同时希望程序尽可能小巧且性能优秀
- App的核心功能是通过与Qwen的图像/视频API交互来完成文生图、图生图以及图中文字翻译等功能
- Qwen AI的URL和APIKey等可以通过读取~/.qwenimage/setting.json来获取
  - 如果用户未提供该配置，则应用程序可以禁用交互方式（未来优化体验）
- 交互的界面以简洁大方为基调，主界面分为左右两个区域：
  - 右侧：交互栏，下方是文本框用户输入文本Prompt内容以及提交按钮，上方是对话内容，API返回的图片或视频也可以通过对话内容的方式呈现给用户，并且用户可以选中资源并选择复制或者下载
    - 文本输入框还要提供按钮用于上传“图生图”所用到的原图片，允许多张图片提交，并在UI上通过布局的顺序或者明确的显示来确认顺序
    - 同时可以支持直接通过Ctrl+V的方式将剪切版中的图像数据（可以通过保存临时目录下的数据来实现）
  - 左侧：历史栏，本身可以折叠，通过点击可以将右侧栏恢复到当时的对话内容

# Setting.json示例

```bash
{
    "providers": {
        "qwen": {
            "apiKey": "sk-xxxxx",
            "models": {
                "qwen-image-max": {
                    "url": "<https://dashscope.aliyuncs.com/api/v1/services/aigc/text2image/image-synthesis>"
                },
                "qwen-image-edit-max": {
                    "url": "<https://dashscope.aliyuncs.com/api/v1/services/aigc/multimodal-generation/generation>"
                },
                "qwen-mt-image": {
                    "url": "<https://dashscope.aliyuncs.com/api/v1/services/aigc/image2image/image-synthesis>"
                }
            }
        }
    }
}
```

### 官方调用示例

文生图示例：

```bash
curl -X POST <https://dashscope.aliyuncs.com/api/v1/services/aigc/text2image/image-synthesis> \\
  -H 'X-DashScope-Async: enable' \\
  -H "Authorization: Bearer $DASHSCOPE_API_KEY" \\
  -H 'Content-Type: application/json' \\
  -d '{
  "model": "qwen-image-plus",
  "input": {
      "prompt": "一副典雅庄重的对联悬挂于厅堂之中，房间是个安静古典的中式布置，桌子上放着一些青花瓷，对联上左书“义本生知人机同道善思新”，右书“通云赋智乾坤启数高志远”， 横批“智启千问”，字体飘逸，在中间挂着一幅中国风的画作，内容是岳阳楼。"
  },
  "parameters": {
      "negative_prompt":" ",
      "size": "1664*928",
      "n": 1,
      "prompt_extend": true,
      "watermark": false
  }
}'
```

图生图示例：

```bash
curl --location '<https://dashscope.aliyuncs.com/api/v1/services/aigc/multimodal-generation/generation>' \\
--header 'Content-Type: application/json' \\
--header "Authorization: Bearer $DASHSCOPE_API_KEY" \\
--data '{
  "model": "qwen-image-edit-max",
  "input": {
      "messages": [
          {
              "role": "user",
              "content": [
                  {
                      "image": "<https://help-static-aliyun-doc.aliyuncs.com/file-manage-files/zh-CN/20250925/thtclx/input1.png>"
                  },
                  {
                      "image": "<https://help-static-aliyun-doc.aliyuncs.com/file-manage-files/zh-CN/20250925/iclsnx/input2.png>"
                  },
                  {
                      "image": "<https://help-static-aliyun-doc.aliyuncs.com/file-manage-files/zh-CN/20250925/gborgw/input3.png>"
                  },
                  {
                      "text": "图1中的女生穿着图2中的黑色裙子按图3的姿势坐下"
                  }
              ]
          }
      ]
  },
  "parameters": {
      "n": 2,
      "negative_prompt": " ",
      "prompt_extend": true,
      "watermark": false,
      "size": "1024*1536"
  }
}'
```

图片翻译示例：

```bash
curl --location '<https://dashscope.aliyuncs.com/api/v1/services/aigc/image2image/image-synthesis>' \\
--header 'X-DashScope-Async: enable' \\
--header "Authorization: Bearer $DASHSCOPE_API_KEY" \\
--header 'Content-Type: application/json' \\
--data '{
  "model": "qwen-mt-image",
  "input": {
      "image_url": "<https://help-static-aliyun-doc.aliyuncs.com/file-manage-files/zh-CN/20250916/ordhsk/1.webp>",
      "source_lang": "zh",
      "target_lang": "en",
      "ext": {
          "config": {
              "imageSegment": false
          }
      }
  }
}'
```


