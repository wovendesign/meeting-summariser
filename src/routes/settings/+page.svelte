<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import * as Card from "$lib/components/ui/card";
  import { toast, Toaster } from "svelte-sonner";
  import { onDestroy, onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  let pythonStatus = $state("Checking...");
  let downloadProgress = $state("");
  let llmProgress = $state("");
  let llmDownloadProgress = $state(0);
  let llmLoadingProgress = $state(0);
  let isDownloadingPython = $state(false);
  let isDownloadingWhisper = $state(false);
  let isTestingLlm = $state(false);

  // LLM Configuration
  let llmConfig = $state({
    use_external_api: true,
    external_endpoint: "http://localhost:11434/v1",
    external_model: "llama3",
  });
  let isSavingConfig = $state(false);
  let unlistenWhisper: UnlistenFn;
  let unlistenPython: UnlistenFn;
  let unlistenLlm: UnlistenFn;
  let unlistenLlmDownload: UnlistenFn;
  let unlistenLlmLoading: UnlistenFn;
  onMount(async () => {
    // Listen for download progress events
    unlistenWhisper = await listen("whisperx-download-progress", (event) => {
      downloadProgress =
        typeof event.payload === "string"
          ? event.payload
          : JSON.stringify(event.payload, null, 2);
      console.log("WhisperX progress:", event.payload);
    });

    unlistenPython = await listen("python-download-progress", (event) => {
      downloadProgress =
        typeof event.payload === "string"
          ? event.payload
          : JSON.stringify(event.payload, null, 2);
      console.log("Python progress:", event.payload);
    });

    // Listen for LLM progress events
    unlistenLlm = await listen("llm-progress", (event) => {
      llmProgress =
        typeof event.payload === "string"
          ? event.payload
          : JSON.stringify(event.payload, null, 2);
      console.log("LLM progress:", event.payload);
    });

    // Listen for LLM download progress
    unlistenLlmDownload = await listen("llm-download-progress", (event) => {
      llmDownloadProgress =
        typeof event.payload === "number" ? event.payload : 0;
      console.log("LLM download progress:", event.payload);
    });

    // Listen for LLM loading progress
    unlistenLlmLoading = await listen("llm-loading-progress", (event) => {
      llmLoadingProgress =
        typeof event.payload === "number" ? event.payload : 0;
      console.log("LLM loading progress:", event.payload);
    });

    // Load LLM configuration
    await loadLlmConfig();
  });

  onDestroy(() => {
    // Clean up listeners if needed
    unlistenWhisper?.();
    unlistenPython?.();
    unlistenLlm?.();
    unlistenLlmDownload?.();
    unlistenLlmLoading?.();
  });

  async function checkPythonStatus() {
    try {
      const status = await invoke("check_whisperx_installation");
      console.log("Python status:", status);
      pythonStatus = "Available";
    } catch (error) {
      toast.error("Error checking Python status: " + error);
      pythonStatus = "Error checking status";
      console.error("Error checking Python status:", error);
    }
  }

  async function downloadPython() {
    try {
      isDownloadingPython = true;
      downloadProgress = "Starting Python download...";

      await invoke("download_python");

      toast.success("Python downloaded successfully!");
      pythonStatus = "Downloaded";
      downloadProgress = "";
    } catch (error) {
      toast.error("Error downloading Python: " + error);
      console.error("Error downloading Python:", error);
      downloadProgress = "";
    } finally {
      isDownloadingPython = false;
    }
  }

  async function loadLlmConfig() {
    try {
      llmConfig = await invoke("get_llm_config");
    } catch (error) {
      toast.error("Error loading LLM config: " + error);
      console.error("Error loading LLM config:", error);
    }
  }
  async function saveLlmConfig() {
    try {
      isSavingConfig = true;
      await invoke("set_llm_config", {
        useExternalApi: llmConfig.use_external_api,
        externalEndpoint: llmConfig.external_endpoint,
        externalModel: llmConfig.external_model,
      });
      toast.success("LLM configuration saved successfully!");
    } catch (error) {
      toast.error("Error saving LLM config: " + error);
      console.error("Error saving LLM config:", error);
    } finally {
      isSavingConfig = false;
    }
  }

  async function testLlmConfig() {
    try {
      isTestingLlm = true;
      llmProgress = "Starting LLM test...";
      llmDownloadProgress = 0;
      llmLoadingProgress = 0;

      // Create a test command that uses the LLM
      const testResult = await invoke("test_llm_connection");

      toast.success("LLM test successful!");
      llmProgress = "";
    } catch (error) {
      toast.error("LLM test failed: " + error);
      console.error("LLM test error:", error);
      llmProgress = "";
    } finally {
      isTestingLlm = false;
    }
  }

  async function downloadWhisper() {
    try {
      isDownloadingWhisper = true;
      downloadProgress = "Starting download...";

      await invoke("download_whisperx");

      toast.success("Whisper downloaded successfully!");
      downloadProgress = "";
    } catch (error) {
      toast.error("Error downloading Whisper: " + error);
      console.error("Error downloading Whisper:", error);
      downloadProgress = "";
    } finally {
      isDownloadingWhisper = false;
    }
  }
</script>

<div class="flex flex-col gap-4 p-8 overflow-y-scroll h-full">
  <Toaster />
  <Button variant="outline" href="/" class="self-start">Back</Button>
  <h1 class="text-3xl font-bold">Settings</h1>

  <!-- LLM Configuration Section -->
  <Card.Root>
    <Card.Header>
      <Card.Title>Language Model Configuration</Card.Title>
      <Card.Description>
        Configure how the application generates summaries and meeting names. You
        can use an external API like Ollama, or fall back to the built-in Kalosm
        model.
      </Card.Description>
    </Card.Header>
    <Card.Content class="space-y-4">
      <div class="space-y-2">
        <Label>LLM Backend</Label>
        <div class="flex gap-2">
          <Button
            variant={llmConfig.use_external_api ? "default" : "outline"}
            onclick={() => (llmConfig.use_external_api = true)}
          >
            External API (Ollama)
          </Button>
          <Button
            variant={!llmConfig.use_external_api ? "default" : "outline"}
            onclick={() => (llmConfig.use_external_api = false)}
          >
            Built-in Kalosm
          </Button>
        </div>
      </div>

      {#if llmConfig.use_external_api}
        <div class="space-y-2">
          <Label for="endpoint">API Endpoint</Label>
          <Input
            id="endpoint"
            bind:value={llmConfig.external_endpoint}
            placeholder="http://localhost:11434/v1"
          />
        </div>

        <div class="space-y-2">
          <Label for="model">Model Name</Label>
          <Input
            id="model"
            bind:value={llmConfig.external_model}
            placeholder="llama3"
          />
        </div>
      {:else}
        <div class="p-3 bg-muted rounded-lg">
          <p class="text-sm text-muted-foreground">
            Using built-in Kalosm model (Phi-3.5 Mini). This will download the
            model on first use (~2GB).
          </p>
        </div>
      {/if}
    </Card.Content>
    <Card.Footer class="flex gap-2">
      <Button onclick={saveLlmConfig} disabled={isSavingConfig || isTestingLlm}>
        {isSavingConfig ? "Saving..." : "Save LLM Configuration"}
      </Button>
      <Button
        variant="outline"
        onclick={testLlmConfig}
        disabled={isSavingConfig || isTestingLlm}
      >
        {isTestingLlm ? "Testing..." : "Test Connection"}
      </Button>
    </Card.Footer>
  </Card.Root>

  {#if llmProgress}
    <div class="mt-4 p-4 bg-muted rounded-lg">
      <h3 class="font-semibold mb-2">LLM Progress:</h3>
      <div
        class="bg-background p-3 rounded border font-mono text-sm max-h-32 overflow-y-auto"
      >
        {llmProgress}
      </div>

      <!-- Download Progress Bar -->
      {#if llmDownloadProgress > 0 && llmDownloadProgress < 100}
        <div class="mt-3">
          <div class="flex justify-between items-center mb-1">
            <span class="text-sm text-muted-foreground">Downloading Model</span>
            <span class="text-sm font-medium">{llmDownloadProgress}%</span>
          </div>
          <!-- <Progress.Root value={llmDownloadProgress} class="h-2" /> -->
        </div>
      {/if}

      <!-- Loading Progress Bar -->
      {#if llmLoadingProgress > 0 && llmLoadingProgress < 100}
        <div class="mt-3">
          <div class="flex justify-between items-center mb-1">
            <span class="text-sm text-muted-foreground">Loading Model</span>
            <span class="text-sm font-medium">{llmLoadingProgress}%</span>
          </div>
          <!-- <Progress.Root value={llmLoadingProgress} class="h-2" /> -->
        </div>
      {/if}

      {#if isTestingLlm}
        <div class="mt-2 flex items-center gap-2">
          <div
            class="animate-spin h-4 w-4 border-2 border-primary border-t-transparent rounded-full"
          ></div>
          <span class="text-sm text-muted-foreground"
            >Testing LLM connection...</span
          >
        </div>
      {/if}
    </div>
  {/if}

  <!-- Speech Recognition Section -->
  <Card.Root>
    <Card.Header>
      <Card.Title>Speech Recognition</Card.Title>
      <Card.Description>
        Python and WhisperX are required for audio transcription.
      </Card.Description>
    </Card.Header>
    <Card.Content>
      <div class="flex gap-2 items-center">
        <p>
          Python status: <span class="p-2 bg-foreground/10 rounded"
            >{pythonStatus}</span
          >
        </p>
        <Button onclick={checkPythonStatus}>Check Again</Button>
      </div>
    </Card.Content>
    <Card.Footer class="flex gap-2">
      <Button
        variant="outline"
        onclick={downloadPython}
        disabled={isDownloadingPython || isDownloadingWhisper}
      >
        {isDownloadingPython ? "Downloading..." : "Download Python"}
      </Button>
      <Button
        variant="outline"
        onclick={downloadWhisper}
        disabled={isDownloadingWhisper || isDownloadingPython}
      >
        {isDownloadingWhisper ? "Downloading..." : "Download Whisper"}
      </Button>
    </Card.Footer>
  </Card.Root>

  {#if downloadProgress}
    <div class="mt-4 p-4 bg-muted rounded-lg">
      <h3 class="font-semibold mb-2">Download Progress:</h3>
      <div
        class="bg-background p-3 rounded border font-mono text-sm max-h-32 overflow-y-auto"
      >
        {downloadProgress}
      </div>
      {#if isDownloadingPython || isDownloadingWhisper}
        <div class="mt-2 flex items-center gap-2">
          <div
            class="animate-spin h-4 w-4 border-2 border-primary border-t-transparent rounded-full"
          ></div>
          <span class="text-sm text-muted-foreground">
            {isDownloadingPython
              ? "Installing Python..."
              : "Installing WhisperX..."}
          </span>
        </div>
      {/if}
    </div>
  {/if}
</div>
