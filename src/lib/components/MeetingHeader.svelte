<script lang="ts">
  import { buttonVariants } from "$lib/components/ui/button/button.svelte";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import Button from "$lib/components/ui/button/button.svelte";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import Ellipsis from "@lucide/svelte/icons/ellipsis";
  import Pen from "@lucide/svelte/icons/pen";
  import FolderClosed from "@lucide/svelte/icons/folder-closed";
  import clsx from "clsx";

  interface Props {
    name: string;
    generatingName: boolean;
    onRevealInFinder?: () => void;
    onRenameMeeting?: (newName: string) => void;
  }

  let { name, generatingName, onRevealInFinder, onRenameMeeting }: Props =
    $props();

  let newMeetingName = $state(name);
  let isRenameDialogOpen = $state(false);

  function handleRenameMeeting() {
    if (
      onRenameMeeting &&
      newMeetingName.trim() &&
      newMeetingName.trim() !== name
    ) {
      onRenameMeeting(newMeetingName.trim());
    }
    isRenameDialogOpen = false;
  }

  function openRenameDialog() {
    newMeetingName = name;
    isRenameDialogOpen = true;
  }
</script>

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
      <DropdownMenu.Item onclick={openRenameDialog}>
        <Pen />
        <span>Rename Meeting</span>
      </DropdownMenu.Item>
      <DropdownMenu.Item onclick={onRevealInFinder}>
        <FolderClosed />
        <span>Reveal in Finder</span>
      </DropdownMenu.Item>
    </DropdownMenu.Content>
  </DropdownMenu.Root>
</div>

<Dialog.Root bind:open={isRenameDialogOpen}>
  <Dialog.Content class="sm:max-w-[425px]">
    <Dialog.Header>
      <Dialog.Title>Rename Meeting</Dialog.Title>
      <Dialog.Description>
        Enter a new name for this meeting.
      </Dialog.Description>
    </Dialog.Header>
    <div class="grid gap-4 py-4">
      <div class="grid grid-cols-4 items-center gap-4">
        <Label for="meetingName" class="text-right">Name</Label>
        <Input
          id="meetingName"
          bind:value={newMeetingName}
          class="col-span-3"
          onkeydown={(e) => {
            if (e.key === "Enter") {
              handleRenameMeeting();
            }
          }}
        />
      </div>
    </div>
    <Dialog.Footer>
      <Button variant="outline" onclick={() => (isRenameDialogOpen = false)}
        >Cancel</Button
      >
      <Button
        type="submit"
        onclick={handleRenameMeeting}
        disabled={!newMeetingName.trim() || newMeetingName.trim() === name}
      >
        Save changes
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
