/**
 * PR 3 — Object360Modal regression tests.
 *
 * Verifies the demo-verified behavior on the shared engine: the modal
 * opens with the record graph, legend layer chips render, clicking a
 * related pill refreshes the inspector, "Show what's around it"
 * re-centers the graph, "Open record" fires, and Close works.
 */
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState, type ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { Object360Modal } from "../components/Object360Modal";
import { api } from "../lib/api";

vi.mock("../lib/api", () => ({
  api: {
    listObjectTypes: vi.fn(),
    listRelatedRecords: vi.fn(),
    listNativeRelatedRecords: vi.fn(),
    listBusinessRules: vi.fn(),
    listWorkflowRules: vi.fn(),
    listAiAgentsScopedToObjectType: vi.fn(),
    getRecordOwner: vi.fn(),
    listOrgUnits: vi.fn(),
    listAuditEvents: vi.fn(),
  },
}));

const objectTypes = [
  { key: "Company", label_singular: "Company", color_index: 1 },
  { key: "Contact", label_singular: "Contact", color_index: 2 },
  { key: "Opportunity", label_singular: "Opportunity", color_index: 3 },
];

const relatedRecords = [
  {
    instance_id: "rel-1",
    relationship_definition_id: "rd1",
    relationship_key: "contacts",
    label: "contacts",
    entity_type: "Contact",
    entity_id: "c2",
    display_name: "Ava Martin",
    status: "active",
    archived: false,
  },
  {
    instance_id: "rel-2",
    relationship_definition_id: "rd2",
    relationship_key: "opportunities",
    label: "opportunities",
    entity_type: "Opportunity",
    entity_id: "o9",
    display_name: "Platform Modernization",
    status: "open",
    archived: false,
  },
];

function mockApi() {
  vi.mocked(api.listObjectTypes).mockResolvedValue(objectTypes as never);
  vi.mocked(api.listRelatedRecords).mockResolvedValue(relatedRecords as never);
  vi.mocked(api.listNativeRelatedRecords).mockResolvedValue([] as never);
  vi.mocked(api.listBusinessRules).mockResolvedValue([{ id: "r1", name: "Credit limit check" }] as never);
  vi.mocked(api.listWorkflowRules).mockResolvedValue([
    { id: "w1", name: "Renewal workflow", is_active: true },
    { id: "w2", name: "Retired workflow", is_active: false },
  ] as never);
  vi.mocked(api.listAiAgentsScopedToObjectType).mockResolvedValue([{ id: "a1", name: "Research Analyst", icon: "🔍" }] as never);
  vi.mocked(api.getRecordOwner).mockResolvedValue({ owning_org_unit_id: "ou1" } as never);
  vi.mocked(api.listOrgUnits).mockResolvedValue([{ id: "ou1", name: "Northstar" }] as never);
  vi.mocked(api.listAuditEvents).mockResolvedValue([
    { id: "e1", summary: "Record created", occurred_at: "2026-01-02T10:00:00Z" },
  ] as never);
}

function Providers({ children }: { children: ReactNode }) {
  const [client] = useState(
    () => new QueryClient({ defaultOptions: { queries: { retry: false, staleTime: Infinity } } }),
  );
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

function inspectorTitle(container: HTMLElement): string | null {
  return container.querySelector(".ontology-inspector-head strong")?.textContent ?? null;
}

describe("Object360Modal", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockApi();
  });

  function open(overrides: Partial<Parameters<typeof Object360Modal>[0]> = {}) {
    const onClose = vi.fn();
    const utils = render(
      <Providers>
        <Object360Modal entityType="Company" entityId="c1" label="BrightPath Logistics" onClose={onClose} {...overrides} />
      </Providers>,
    );
    return { onClose, ...utils };
  }

  it("opens with the record title, breadcrumb, and graph", async () => {
    const { container } = open();
    expect(await screen.findByText("Object 360 · BrightPath Logistics")).toBeInTheDocument();
    // Breadcrumb: Company › BrightPath Logistics › Relationship neighborhood
    expect(screen.getByText(/Relationship neighborhood/)).toBeInTheDocument();
    // Root pill + related pills render on the shared engine.
    await waitFor(() => {
      expect(container.querySelector('[data-node-key="record:Company:c1"]')).not.toBeNull();
    });
    expect(container.querySelector('[data-node-key="record:Contact:c2"]')).not.toBeNull();
    expect(container.querySelector('[data-node-key="record:Opportunity:o9"]')).not.toBeNull();
  });

  it("renders legend layer chips for every neighbor kind", async () => {
    open();
    // Wait for the related records to load before reading the legend.
    expect((await screen.findAllByText("Ava Martin")).length).toBeGreaterThan(0);
    const legend = await screen.findByRole("group", { name: "Graph layers" });
    const text = legend.textContent ?? "";
    for (const chip of ["Company", "Contact", "Opportunity", "Business Rules", "Workflows", "AI Agents", "Business Unit"]) {
      expect(text).toContain(chip);
    }
  });

  it("clicking a related pill refreshes the inspector", async () => {
    const { container } = open();
    await screen.findByText("Object 360 · BrightPath Logistics");
    await waitFor(() => {
      expect(container.querySelector('[data-node-key="record:Contact:c2"]')).not.toBeNull();
    });
    // Inspector starts on the root record.
    expect(inspectorTitle(container)).toBe("BrightPath Logistics");

    fireEvent.click(container.querySelector('[data-node-key="record:Contact:c2"]')!);

    await waitFor(() => {
      expect(inspectorTitle(container)).toBe("Ava Martin");
    });
    // Subtitle reflects the inspected record's type.
    const head = container.querySelector(".ontology-inspector-head")!;
    expect(within(head as HTMLElement).getByText("Contact")).toBeInTheDocument();
    // The inspected pill gets the ring.
    expect(container.querySelector('[data-node-key="record:Contact:c2"]')).toHaveClass("og-pill-inspecting");
  });

  it("shows related-record groups, rules, workflows, agents, and activity for the root", async () => {
    const { container } = open();
    await screen.findByText("Object 360 · BrightPath Logistics");
    // Wait for the related records to load into the inspector.
    expect((await screen.findAllByText("Ava Martin")).length).toBeGreaterThan(0);
    const groups = container.querySelector(".ontology-inspector-groups")!;
    const text = groups.textContent ?? "";
    expect(text).toContain("Ava Martin");
    expect(text).toContain("Business Unit");
    expect(text).toContain("Northstar");
    expect(text).toContain("Credit limit check");
    expect(text).toContain("Renewal workflow");
    expect(text).toContain("Research Analyst");
    expect(text).toContain("Record created");
    // Inactive workflows are excluded.
    expect(text).not.toContain("Retired workflow");
  });

  it("'Show what's around it' re-centers the graph on the inspected record", async () => {
    const { container } = open();
    await screen.findByText("Object 360 · BrightPath Logistics");
    await waitFor(() => {
      expect(container.querySelector('[data-node-key="record:Contact:c2"]')).not.toBeNull();
    });
    fireEvent.click(container.querySelector('[data-node-key="record:Contact:c2"]')!);
    await waitFor(() => {
      expect(inspectorTitle(container)).toBe("Ava Martin");
    });
    fireEvent.click(screen.getByRole("button", { name: "Show what's around it" }));
    expect(await screen.findByText("Object 360 · Ava Martin")).toBeInTheDocument();
  });

  it("fires onOpenRecord for the inspected record", async () => {
    const onOpenRecord = vi.fn();
    const { container } = open({ onOpenRecord });
    await screen.findByText("Object 360 · BrightPath Logistics");
    await waitFor(() => {
      expect(container.querySelector('[data-node-key="record:Contact:c2"]')).not.toBeNull();
    });
    fireEvent.click(container.querySelector('[data-node-key="record:Contact:c2"]')!);
    await waitFor(() => {
      expect(inspectorTitle(container)).toBe("Ava Martin");
    });
    fireEvent.click(screen.getByRole("button", { name: "Open record" }));
    expect(onOpenRecord).toHaveBeenCalledWith("Contact", "c2");
  });

  it("closes via the Close button and overlay click", async () => {
    const { container, onClose } = open();
    await screen.findByText("Object 360 · BrightPath Logistics");
    fireEvent.click(screen.getByRole("button", { name: "Close" }));
    expect(onClose).toHaveBeenCalledTimes(1);
    fireEvent.click(container.querySelector(".modal-overlay")!);
    expect(onClose).toHaveBeenCalledTimes(2);
  });
});
