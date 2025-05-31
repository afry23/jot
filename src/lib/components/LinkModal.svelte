<script lang="ts">
    import { createEventDispatcher, onMount } from "svelte";
    export let initialHref = "";
    export let initialText = "";
    const dispatch = createEventDispatcher();
    let href = initialHref;
    let text = initialText;
    let hrefInput: HTMLInputElement;

    function submit() {
        dispatch("submit", { href, text });
    }
    function cancel() {
        dispatch("cancel");
    }
    onMount(() => {
        hrefInput?.focus();
    });
</script>

<div class="modal-backdrop" on:click={cancel}></div>
<div class="modal">
    <form on:submit|preventDefault={submit}>
        <label>
            Link URL:
            <input
                bind:this={hrefInput}
                bind:value={href}
                type="url"
                required
            />
        </label>
        <label>
            Link Text:
            <input bind:value={text} type="text" required />
        </label>
        <div class="actions">
            <button type="submit">OK</button>
            <button type="button" on:click={cancel}>Cancel</button>
        </div>
    </form>
</div>

<style>
    .modal-backdrop {
        position: fixed;
        inset: 0;
        background: rgba(0, 0, 0, 0.3);
        z-index: 1000;
    }
    .modal {
        position: fixed;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
        background: var(--background, #fff);
        color: var(--foreground, #222);
        padding: 2em;
        border-radius: 8px;
        z-index: 1001;
        min-width: 320px;
        box-shadow: 0 8px 32px rgba(0, 0, 0, 0.18);
    }
    .actions {
        margin-top: 1em;
        display: flex;
        gap: 1em;
        justify-content: flex-end;
    }
    label {
        display: block;
        margin-bottom: 1em;
    }
    input[type="url"],
    input[type="text"] {
        width: 100%;
        padding: 0.5em;
        margin-top: 0.25em;
        border: 1px solid #ccc;
        border-radius: 4px;
    }
</style>
