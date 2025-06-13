export interface ProgressState {
  currentChunk: number;
  totalChunks: number;
  isActive: boolean;
}

export function useProgressTracking() {
  // Transcription progress tracking
  let transcriptionProgress = $state<ProgressState>({
    currentChunk: 0,
    totalChunks: 0,
    isActive: false,
  });

  // Audio splitting progress tracking
  let audioSplittingProgress = $state<ProgressState>({
    currentChunk: 0,
    totalChunks: 0,
    isActive: false,
  });

  // Summarization progress tracking
  let summarizationProgress = $state<ProgressState>({
    currentChunk: 0,
    totalChunks: 0,
    isActive: false,
  });

  function resetTranscriptionProgress() {
    transcriptionProgress.isActive = false;
    transcriptionProgress.currentChunk = 0;
    transcriptionProgress.totalChunks = 0;
  }

  function resetAudioSplittingProgress() {
    audioSplittingProgress.isActive = false;
    audioSplittingProgress.currentChunk = 0;
    audioSplittingProgress.totalChunks = 0;
  }

  function resetSummarizationProgress() {
    summarizationProgress.isActive = false;
    summarizationProgress.currentChunk = 0;
    summarizationProgress.totalChunks = 0;
  }

  function startTranscription(totalChunks: number) {
    transcriptionProgress.totalChunks = totalChunks;
    transcriptionProgress.currentChunk = 0;
    transcriptionProgress.isActive = true;
    audioSplittingProgress.isActive = false; // Audio splitting is done
  }

  function updateTranscriptionProgress(chunkIndex: number) {
    transcriptionProgress.currentChunk = chunkIndex + 1; // +1 because backend sends 0-based index
  }

  function startAudioSplitting(totalChunks: number) {
    audioSplittingProgress.totalChunks = totalChunks;
    audioSplittingProgress.currentChunk = 0;
    audioSplittingProgress.isActive = true;
  }

  function updateAudioSplittingProgress(chunkIndex: number) {
    audioSplittingProgress.currentChunk = chunkIndex + 1; // +1 because backend sends 0-based index
  }

  function startSummarization(totalChunks: number) {
    summarizationProgress.totalChunks = totalChunks;
    summarizationProgress.currentChunk = 0;
    summarizationProgress.isActive = true;
  }

  function updateSummarizationProgress(chunkIndex: number) {
    summarizationProgress.currentChunk = chunkIndex + 1; // +1 because backend sends 0-based index
  }

  return {
    // State - return direct access to reactive state
    get transcriptionProgress() { return transcriptionProgress; },
    get audioSplittingProgress() { return audioSplittingProgress; },
    get summarizationProgress() { return summarizationProgress; },

    // Actions
    resetTranscriptionProgress,
    resetAudioSplittingProgress,
    resetSummarizationProgress,
    startTranscription,
    updateTranscriptionProgress,
    startAudioSplitting,
    updateAudioSplittingProgress,
    startSummarization,
    updateSummarizationProgress,
  };
}
