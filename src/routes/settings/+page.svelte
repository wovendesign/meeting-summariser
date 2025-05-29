<script>
  import { Button } from "$lib/components/ui/button";
  import { toast, Toaster } from "svelte-sonner";
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  let pythonStatus = $state("Checking...");

  onMount(async () => {});

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
    <!-- <Button variant="outline" onclick={downloadPython}>Download Python</Button> -->
  </div>
</div>
