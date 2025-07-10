import { type LoaderFunction } from "react-router";
import { useLoaderData, useParams } from "react-router";

export const loader: LoaderFunction = async ({ params }) => {
  const tickets = await fetchTickets(params.projectId);
  return { tickets };
};

export default function ProjectTickets() {
  const { tickets } = useLoaderData<typeof loader>();
  const params = useParams();

  return (
    <div>
      <h2>Tickets</h2>
      <div className="tickets-list">
        {tickets.map(ticket => (
          <div key={ticket.id} className="ticket-card">
            <h3>{ticket.title}</h3>
            <p>{ticket.status}</p>
          </div>
        ))}
      </div>
    </div>
  );
}