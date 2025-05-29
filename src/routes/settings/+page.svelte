<script>
  import { Button } from "$lib/components/ui/button";
  import { toast, Toaster } from "svelte-sonner";
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  let pythonStatus = $state("Checking...");

  onMount(async () => {});

  async function checkPythonStatus() {
    try {
      const status = await invoke("check_python_installation");
      console.log("Python status:", status);
      pythonStatus = status ? "Running" : "Not Running";
    } catch (error) {
      toast.error("Error checking Python status: " + error);
      pythonStatus = "Error checking status";
      console.error("Error checking Python status:", error);
    }
  }

  async function downloadPython() {
    try {
      invoke("install_python")
        .then(() => {
          toast.success("Python download initiated");
        })
        .catch((error) => {
          toast.error("Error downloading Python: " + error.message);
          console.error("Error downloading Python:", error);
        });
    } catch (error) {
      toast.error("Error opening Python download page");
      console.error("Error opening Python download page:", error);
    }
  }

  async function transcribeAudio() {
    try {
      const result = await invoke("transcribe");
      toast.success("Transcription completed: " + result);
    } catch (error) {
      toast.error("Error during transcription: " + error);
      console.error("Error during transcription:", error);
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
    <Button variant="outline" onclick={downloadPython}>Download Python</Button>
  </div>
  <Button onclick={transcribeAudio}>Transcribe Audio</Button>
</div>
