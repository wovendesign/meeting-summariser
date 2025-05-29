<script lang="ts">
  import Recorder from "$lib/components/Recorder.svelte";
  import { Button } from "$lib/components/ui/button";
</script>

<div class="flex flex-col gap-4 p-8">
  <Button variant="outline" href="/" class="self-start">Back</Button>

  <h2 class="text-2xl font-bold">New Meeting</h2>

  <p>Start a new meeting recording:</p>
  <Recorder />
</div>
