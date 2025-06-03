<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { toast, Toaster } from "svelte-sonner";
  import { onDestroy, onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
    import type { UnlistenFn } from "@tauri-apps/api/event";

  let pythonStatus = $state("Checking...");
  let downloadProgress = $state("");
  let isDownloadingPython = $state(false);
  let isDownloadingWhisper = $state(false);

  let unlistenWhisper: UnlistenFn;
  let unlistenPython: UnlistenFn;

  onMount(async () => {
    // Listen for download progress events
    unlistenWhisper = await listen("whisperx-download-progress", (event) => {
      downloadProgress = typeof event.payload === "string" ? event.payload : JSON.stringify(event.payload, null, 2);
      console.log("WhisperX progress:", event.payload);
    });

    unlistenPython = await listen("python-download-progress", (event) => {
      downloadProgress = typeof event.payload === "string" ? event.payload : JSON.stringify(event.payload, null, 2);
      console.log("Python progress:", event.payload);
    });

    // Clean up listeners when component is destroyed
    // return () => {
    //   unlistenWhisper();
    //   unlistenPython();
    // };
  });

  onDestroy(() => {
    // Clean up listeners if needed
    unlistenWhisper();
    unlistenPython();
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

<div class="container flex flex-col gap-4 p-4">
  <Toaster />
  <Button variant="outline" href="/" class="self-start">Back</Button>
  <h1>Settings</h1>

  <div class="flex gap-2">
    <p>
      Python status: <span class="p-2 bg-foreground/10 rounded"
        >{pythonStatus}</span
      >
    </p>
    <Button onclick={checkPythonStatus}>Check Again</Button>
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
  </div>

  {#if downloadProgress}
    <div class="mt-4 p-4 bg-muted rounded-lg">
      <h3 class="font-semibold mb-2">Download Progress:</h3>
      <div class="bg-background p-3 rounded border font-mono text-sm max-h-32 overflow-y-auto">
        {downloadProgress}
      </div>
      {#if isDownloadingPython || isDownloadingWhisper}
        <div class="mt-2 flex items-center gap-2">
          <div class="animate-spin h-4 w-4 border-2 border-primary border-t-transparent rounded-full"></div>
          <span class="text-sm text-muted-foreground">
            {isDownloadingPython ? "Installing Python..." : "Installing WhisperX..."}
          </span>
        </div>
      {/if}
    </div>
  {/if}
</div>
