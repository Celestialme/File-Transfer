<script>
	// Svelte 5 runes

	let color = '#4ade80'; // Tailwind's green-400
	let height = '1rem';
	let showLabel = false;

	let { progress } = $props();

	// Animate towards target progress
	$effect(() => {
		const step = () => {
			if (Math.abs(progress - progress) < 0.1) {
				progress = progress;
				return;
			}
			progress += (progress - progress) * 0.1;
			requestAnimationFrame(step);
		};
		step();
	});
</script>

<div class="progress-container" style:height>
	<div class="progress-fill" style:width={`${progress}%`} style:background-color={color} />
</div>

{#if showLabel}
	<div class="label">{Math.round(progress)}%</div>
{/if}

<style>
	.progress-container {
		flex-grow: 1;
		width: 100%;
		background-color: #b5b5b5;
		border-radius: 9999px;
		overflow: hidden;
	}

	.progress-fill {
		height: 100%;
		border-radius: 9999px;
		transition: width 0.3s ease-in-out;
	}

	.label {
		text-align: center;
		margin-top: 0.25rem;
		font-size: 0.875rem;
		color: #374151; /* Tailwind gray-700 */
	}
</style>
