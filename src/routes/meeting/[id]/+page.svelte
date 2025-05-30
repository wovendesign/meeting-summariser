<script lang="ts">
  import SpeakerNaming from "$lib/components/SpeakerNaming.svelte";
  import Clipboard from "@lucide/svelte/icons/clipboard";

  let { data }: PageProps = $props();

  import { marked } from "marked";
  import type { PageProps } from "./$types";
  import * as Card from "$lib/components/ui/card/index.js";
  import Button, {
    buttonVariants,
  } from "$lib/components/ui/button/button.svelte";
  import { Textarea } from "$lib/components/ui/textarea";
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { toast, Toaster } from "svelte-sonner";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import RefreshCcw from "@lucide/svelte/icons/refresh-ccw";
  import Ellipsis from "@lucide/svelte/icons/ellipsis";
  import Pen from "@lucide/svelte/icons/pen";
  import clsx from "clsx";
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";

  let transcriptContent = $state("");
  let transcriptJsonContent: string | null = $state(null);
  let summaryContent: string | null = $state("");
  let savingTranscript = $state(false);
  let saveStatus = $state("");
  let loadingSummary = $state(false);

  let name = $state(`Meeting ${data.id}`);
  let generatingName = $state(false);

  let meetingMetadata: {
    name?: string;
  } = $state({});

  $effect(() => {
    if (meetingMetadata.name) {
      name = meetingMetadata.name;
    }
  });

  let audio: ArrayBuffer | null = $state(null);
  let audioURL = $derived.by(() => {
    if (!audio) return "";
    const blob = new Blob([audio], { type: "audio/wav" });
    return URL.createObjectURL(blob);
  });

  async function saveTranscript() {
    // savingTranscript = true;
    // saveStatus = '';
    // try {
    // 	const res = await fetch('/api/transcript/update', {
    // 		method: 'POST',
    // 		headers: { 'Content-Type': 'application/json' },
    // 		body: JSON.stringify({ filepath: data.txtPath, content: transcriptContent })
    // 	});
    // 	saveStatus = res.ok ? 'Transcript saved' : `Save failed: ${res.statusText}`;
    // } catch (err) {
    // 	saveStatus = `Error: ${err instanceof Error ? err.message : err}`;
    // } finally {
    // 	savingTranscript = false;
    // }
  }

  async function regenerateSummary() {
    try {
      summaryContent = await invoke("generate_summary", {
        meetingId: data.id,
      });
      await getMeetingMetadata();
    } catch (error) {
      console.error("Error regenerating summary:", error);
      // Handle error appropriately, e.g., show a toast notification
      toast.error("Error regenerating summary: " + error);
      saveStatus = "Error regenerating summary";
      return;
    }
  }

  const markdownContent = $derived.by(() => {
    if (!summaryContent) return "";
    const realNewLinesContent = summaryContent.replace(/\n/g, "\n");
    return marked(realNewLinesContent, {
      gfm: true,
      breaks: true,
    });
  });

  onMount(async () => {
    await getTranscript();
    await getSummary();
    await getAudio();
    await getTranscriptJson();
    await getMeetingMetadata();
  });

  async function getTranscript() {
    try {
      transcriptContent = await invoke("get_meeting_transcript", {
        meetingId: data.id,
      });
    } catch (error) {
      console.error("Error fetching transcript:", error);
      transcriptContent = "Error fetching transcript";
    }
  }

  async function getTranscriptJson() {
    try {
      transcriptJsonContent = await invoke("get_meeting_transcript_json", {
        meetingId: data.id,
      });
    } catch (error) {
      console.error("Error fetching transcript:", error);
      transcriptJsonContent = null;
    }
  }

  async function getSummary() {
    try {
      summaryContent = await invoke("get_meeting_summary", {
        meetingId: data.id,
      });
    } catch (error) {
      console.error("Error fetching summary:", error);
      summaryContent = null;
    }
  }

  async function getAudio() {
    try {
      audio = await invoke("get_meeting_audio", { meetingId: data.id });
    } catch (error) {
      console.error("Error fetching audio:", error);
      audio = null;
    }
  }

  async function getMeetingMetadata() {
    try {
      meetingMetadata = await invoke("get_meeting_metadata", {
        meetingId: data.id,
      });
    } catch (error) {
      console.error("Error fetching meeting metadata:", error);
      meetingMetadata = {};
    }
  }

  async function generateMeetingName() {
    generatingName = true;
    try {
      name = await invoke("generate_meeting_name", { meetingId: data.id });
      console.log(name);
      generatingName = false;
    } catch (error) {
      toast.error("Error generating meeting name: " + error);
      generatingName = false;
    }
  }
</script>

<Toaster />
<div class="flex flex-col gap-4 p-8 overflow-y-scroll h-full">
  <Button variant="outline" href="/" class="self-start">Back</Button>
  <div class="flex items-center justify-between">
    <h2 class={clsx("text-2xl font-bold", generatingName && "animate-pulse")}>
      {name}
    </h2>
    <DropdownMenu.Root>
      <DropdownMenu.Trigger
        class={buttonVariants({ variant: "outline", size: "icon" })}
      >
        <Ellipsis />
      </DropdownMenu.Trigger>
      <DropdownMenu.Content class="w-56 mr-4">
        <DropdownMenu.Item>
          <Pen />
          <span>Rename Meeting</span>
          <Dialog.Root>
            <Dialog.Trigger class={buttonVariants({ variant: "outline" })}
              >Edit Profile</Dialog.Trigger
            >
            <Dialog.Content class="sm:max-w-[425px]">
              <Dialog.Header>
                <Dialog.Title>Edit profile</Dialog.Title>
                <Dialog.Description>
                  Make changes to your profile here. Click save when you're
                  done.
                </Dialog.Description>
              </Dialog.Header>
              <div class="grid gap-4 py-4">
                <div class="grid grid-cols-4 items-center gap-4">
                  <Label for="name" class="text-right">Name</Label>
                  <Input id="name" value="Pedro Duarte" class="col-span-3" />
                </div>
                <div class="grid grid-cols-4 items-center gap-4">
                  <Label for="username" class="text-right">Username</Label>
                  <Input id="username" value="@peduarte" class="col-span-3" />
                </div>
              </div>
              <Dialog.Footer>
                <Button type="submit">Save changes</Button>
              </Dialog.Footer>
            </Dialog.Content>
          </Dialog.Root>
        </DropdownMenu.Item>
        <DropdownMenu.Item onclick={generateMeetingName}>
          <RefreshCcw />
          <span>Re-Generate Name</span>
        </DropdownMenu.Item>
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  </div>

  <Card.Root>
    <Card.Header class="flex items-center justify-between">
      <Card.Title>Recorded Meeting</Card.Title>
      <DropdownMenu.Root>
        <DropdownMenu.Trigger
          class={buttonVariants({ variant: "outline", size: "icon" })}
        >
          <Ellipsis />
        </DropdownMenu.Trigger>
        <DropdownMenu.Content class="w-56 mr-4">
          <DropdownMenu.Item
            onclick={async () => {
              try {
                await invoke("transcribe", { meetingId: data.id }).then(() => {
                  console.log("Transcription finished successfully");
                });
              } catch (err) {
                console.error("Error transcribing audio:", err);
              }
            }}
          >
            <RefreshCcw />
            <span>Transcribe Audio</span>
          </DropdownMenu.Item>
        </DropdownMenu.Content>
      </DropdownMenu.Root>
    </Card.Header>
    <Card.Content>
      <audio src={audioURL} controls class="w-full"></audio>
    </Card.Content>
  </Card.Root>

  <section>
    {#if saveStatus}
      <p>{saveStatus}</p>
    {/if}

    <Card.Root>
      <Card.Header>
        <Card.Title>Transcript</Card.Title>
      </Card.Header>
      <Card.Content>
        <Textarea
          bind:value={transcriptContent}
          placeholder="Edit the Transcript"
        />
      </Card.Content>
      <Card.Footer class="flex gap-2">
        <Button onclick={saveTranscript} disabled={savingTranscript}>
          {savingTranscript ? "Saving..." : "Save Transcript"}
        </Button>
      </Card.Footer>
    </Card.Root>
  </section>

  <section>
    <SpeakerNaming {audioURL} json={transcriptJsonContent} />
  </section>

  <section>
    <Card.Root>
      <Card.Header>
        <Card.Title>Transcription Summary</Card.Title>
      </Card.Header>
      <Card.Content class="prose prose-invert mx-auto">
        {#if summaryContent}
          {@html markdownContent}
        {:else}
          <p>No summary available.</p>
        {/if}
      </Card.Content>
      <Card.Footer class="flex gap-2">
        <Button>
          <Clipboard class="mr-2 size-4" />
          Copy Summary
        </Button>
        <Button onclick={regenerateSummary} disabled={loadingSummary}>
          {loadingSummary ? "Regenerating..." : "Regenerate Summary"}
        </Button>
      </Card.Footer>
    </Card.Root>
  </section>
</div>
