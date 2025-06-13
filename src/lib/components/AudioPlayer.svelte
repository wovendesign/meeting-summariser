<script lang="ts">
  import * as Card from "$lib/components/ui/card/index.js";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import { buttonVariants } from "$lib/components/ui/button/button.svelte";
  import Ellipsis from "@lucide/svelte/icons/ellipsis";
  import RefreshCcw from "@lucide/svelte/icons/refresh-ccw";
  import clsx from "clsx";

  interface Props {
    audioURL: string;
    onTranscribe?: () => void;
  }

  let { audioURL, onTranscribe }: Props = $props();
</script>

<Card.Root class="group">
  <Card.Header class="flex items-center justify-between">
    <Card.Title>Recorded Meeting</Card.Title>
    <DropdownMenu.Root>
      <DropdownMenu.Trigger
        class={clsx(buttonVariants({ variant: "outline", size: "icon" }))}
      >
        <Ellipsis />
      </DropdownMenu.Trigger>
      <DropdownMenu.Content class="w-56 mr-4">
        <DropdownMenu.Item onclick={onTranscribe}>
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
