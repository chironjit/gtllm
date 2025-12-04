# Get remaining credits

GET https://openrouter.ai/api/v1/credits

Get total credits purchased and used for the authenticated user

Reference: https://openrouter.ai/docs/api/api-reference/credits/get-credits

## OpenAPI Specification

```yaml
openapi: 3.1.1
info:
  title: Get remaining credits
  version: endpoint_credits.getCredits
paths:
  /credits:
    get:
      operationId: get-credits
      summary: Get remaining credits
      description: Get total credits purchased and used for the authenticated user
      tags:
        - - subpackage_credits
      parameters:
        - name: Authorization
          in: header
          description: API key as bearer token in Authorization header
          required: true
          schema:
            type: string
      responses:
        '200':
          description: Returns the total credits purchased and used
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Credits_getCredits_Response_200'
        '401':
          description: Unauthorized - Authentication required or invalid credentials
          content: {}
        '403':
          description: Forbidden - Only provisioning keys can fetch credits
          content: {}
        '500':
          description: Internal Server Error - Unexpected server error
          content: {}
components:
  schemas:
    CreditsGetResponsesContentApplicationJsonSchemaData:
      type: object
      properties:
        total_credits:
          type: number
          format: double
        total_usage:
          type: number
          format: double
      required:
        - total_credits
        - total_usage
    Credits_getCredits_Response_200:
      type: object
      properties:
        data:
          $ref: >-
            #/components/schemas/CreditsGetResponsesContentApplicationJsonSchemaData
      required:
        - data

```

# List all models and their properties

GET https://openrouter.ai/api/v1/models

Reference: https://openrouter.ai/docs/api/api-reference/models/get-models

## OpenAPI Specification

```yaml
openapi: 3.1.1
info:
  title: List all models and their properties
  version: endpoint_models.getModels
paths:
  /models:
    get:
      operationId: get-models
      summary: List all models and their properties
      tags:
        - - subpackage_models
      parameters:
        - name: category
          in: query
          required: false
          schema:
            type: string
        - name: supported_parameters
          in: query
          required: false
          schema:
            type: string
        - name: use_rss
          in: query
          required: false
          schema:
            type: string
        - name: use_rss_chat_links
          in: query
          required: false
          schema:
            type: string
        - name: Authorization
          in: header
          description: API key as bearer token in Authorization header
          required: true
          schema:
            type: string
      responses:
        '200':
          description: Returns a list of models or RSS feed
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ModelsListResponse'
        '400':
          description: Bad Request - Invalid request parameters
          content: {}
        '500':
          description: Internal Server Error
          content: {}
components:
  schemas:
    BigNumberUnion:
      oneOf:
        - type: number
          format: double
        - type: string
        - type: number
          format: double
    PublicPricing:
      type: object
      properties:
        prompt:
          $ref: '#/components/schemas/BigNumberUnion'
        completion:
          $ref: '#/components/schemas/BigNumberUnion'
        request:
          $ref: '#/components/schemas/BigNumberUnion'
        image:
          $ref: '#/components/schemas/BigNumberUnion'
        image_token:
          $ref: '#/components/schemas/BigNumberUnion'
        image_output:
          $ref: '#/components/schemas/BigNumberUnion'
        audio:
          $ref: '#/components/schemas/BigNumberUnion'
        input_audio_cache:
          $ref: '#/components/schemas/BigNumberUnion'
        web_search:
          $ref: '#/components/schemas/BigNumberUnion'
        internal_reasoning:
          $ref: '#/components/schemas/BigNumberUnion'
        input_cache_read:
          $ref: '#/components/schemas/BigNumberUnion'
        input_cache_write:
          $ref: '#/components/schemas/BigNumberUnion'
        discount:
          type: number
          format: double
      required:
        - prompt
        - completion
    ModelGroup:
      type: string
      enum:
        - value: Router
        - value: Media
        - value: Other
        - value: GPT
        - value: Claude
        - value: Gemini
        - value: Grok
        - value: Cohere
        - value: Nova
        - value: Qwen
        - value: Yi
        - value: DeepSeek
        - value: Mistral
        - value: Llama2
        - value: Llama3
        - value: Llama4
        - value: PaLM
        - value: RWKV
        - value: Qwen3
    ModelArchitectureInstructType:
      type: string
      enum:
        - value: none
        - value: airoboros
        - value: alpaca
        - value: alpaca-modif
        - value: chatml
        - value: claude
        - value: code-llama
        - value: gemma
        - value: llama2
        - value: llama3
        - value: mistral
        - value: nemotron
        - value: neural
        - value: openchat
        - value: phi3
        - value: rwkv
        - value: vicuna
        - value: zephyr
        - value: deepseek-r1
        - value: deepseek-v3.1
        - value: qwq
        - value: qwen3
    InputModality:
      type: string
      enum:
        - value: text
        - value: image
        - value: file
        - value: audio
        - value: video
    OutputModality:
      type: string
      enum:
        - value: text
        - value: image
        - value: embeddings
    ModelArchitecture:
      type: object
      properties:
        tokenizer:
          $ref: '#/components/schemas/ModelGroup'
        instruct_type:
          oneOf:
            - $ref: '#/components/schemas/ModelArchitectureInstructType'
            - type: 'null'
        modality:
          type:
            - string
            - 'null'
        input_modalities:
          type: array
          items:
            $ref: '#/components/schemas/InputModality'
        output_modalities:
          type: array
          items:
            $ref: '#/components/schemas/OutputModality'
      required:
        - modality
        - input_modalities
        - output_modalities
    TopProviderInfo:
      type: object
      properties:
        context_length:
          type:
            - number
            - 'null'
          format: double
        max_completion_tokens:
          type:
            - number
            - 'null'
          format: double
        is_moderated:
          type: boolean
      required:
        - is_moderated
    PerRequestLimits:
      type: object
      properties:
        prompt_tokens:
          type: number
          format: double
        completion_tokens:
          type: number
          format: double
      required:
        - prompt_tokens
        - completion_tokens
    Parameter:
      type: string
      enum:
        - value: temperature
        - value: top_p
        - value: top_k
        - value: min_p
        - value: top_a
        - value: frequency_penalty
        - value: presence_penalty
        - value: repetition_penalty
        - value: max_tokens
        - value: logit_bias
        - value: logprobs
        - value: top_logprobs
        - value: seed
        - value: response_format
        - value: structured_outputs
        - value: stop
        - value: tools
        - value: tool_choice
        - value: parallel_tool_calls
        - value: include_reasoning
        - value: reasoning
        - value: web_search_options
        - value: verbosity
    DefaultParameters:
      type: object
      properties:
        temperature:
          type:
            - number
            - 'null'
          format: double
        top_p:
          type:
            - number
            - 'null'
          format: double
        frequency_penalty:
          type:
            - number
            - 'null'
          format: double
    Model:
      type: object
      properties:
        id:
          type: string
        canonical_slug:
          type: string
        hugging_face_id:
          type:
            - string
            - 'null'
        name:
          type: string
        created:
          type: number
          format: double
        description:
          type: string
        pricing:
          $ref: '#/components/schemas/PublicPricing'
        context_length:
          type:
            - number
            - 'null'
          format: double
        architecture:
          $ref: '#/components/schemas/ModelArchitecture'
        top_provider:
          $ref: '#/components/schemas/TopProviderInfo'
        per_request_limits:
          $ref: '#/components/schemas/PerRequestLimits'
        supported_parameters:
          type: array
          items:
            $ref: '#/components/schemas/Parameter'
        default_parameters:
          $ref: '#/components/schemas/DefaultParameters'
      required:
        - id
        - canonical_slug
        - name
        - created
        - pricing
        - context_length
        - architecture
        - top_provider
        - per_request_limits
        - supported_parameters
        - default_parameters
    ModelsListResponseData:
      type: array
      items:
        $ref: '#/components/schemas/Model'
    ModelsListResponse:
      type: object
      properties:
        data:
          $ref: '#/components/schemas/ModelsListResponseData'
      required:
        - data

```

# Get total count of available models

GET https://openrouter.ai/api/v1/models/count

Reference: https://openrouter.ai/docs/api/api-reference/models/list-models-count

## OpenAPI Specification

```yaml
openapi: 3.1.1
info:
  title: Get total count of available models
  version: endpoint_models.listModelsCount
paths:
  /models/count:
    get:
      operationId: list-models-count
      summary: Get total count of available models
      tags:
        - - subpackage_models
      parameters:
        - name: Authorization
          in: header
          description: API key as bearer token in Authorization header
          required: true
          schema:
            type: string
      responses:
        '200':
          description: Returns the total count of available models
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ModelsCountResponse'
        '500':
          description: Internal Server Error
          content: {}
components:
  schemas:
    ModelsCountResponseData:
      type: object
      properties:
        count:
          type: number
          format: double
      required:
        - count
    ModelsCountResponse:
      type: object
      properties:
        data:
          $ref: '#/components/schemas/ModelsCountResponseData'
      required:
        - data

```
